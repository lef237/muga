package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"time"
)

type point struct{ x, y int64 }
type step struct {
	add   bool
	value int64
}

func fib(n int64) int64 {
	if n < 2 {
		return n
	}
	return fib(n-1) + fib(n-2)
}

func run(name, data string) (int64, error) {
	switch name {
	case "cpu_loop":
		var sum int64
		for i := int64(0); i < 200000; i++ {
			sum = (sum*33 + i*17 + 3) % 1000003
		}
		return sum, nil
	case "cpu_recursion":
		return fib(22), nil
	case "cpu_records":
		p := point{1, 2}
		for i := int64(0); i < 10000; i++ {
			x := (p.x*33 + i) % 1000003
			y := (p.y*17 + x) % 1000003
			p = point{x, y}
		}
		return p.x + p.y, nil
	case "cpu_enums":
		var total int64
		state := int64(1)
		for i := int64(0); i < 20000; i++ {
			state = (state*33 + i) % 1000003
			s := step{i%2 == 0, state}
			if s.add {
				total += s.value
			} else {
				total -= s.value
			}
		}
		return total, nil
	case "string":
		text := ""
		for i := 0; i < 300; i++ {
			text += "alpha,beta,gamma|"
		}
		return int64(len(strings.Split(text, "|")) + len(text)), nil
	case "list":
		values := make([]int64, 0)
		for i := int64(0); i < 800; i++ {
			values = append(values, i)
		}
		var sum int64
		for _, value := range values {
			sum += value
		}
		return sum, nil
	case "map":
		items := make(map[string]int64)
		for i := int64(0); i < 300; i++ {
			items[strconv.FormatInt(i, 10)] = i
		}
		var total int64
		for i := int64(0); i < 300; i++ {
			total += items[strconv.FormatInt(i, 10)]
		}
		return total, nil
	case "json":
		content, err := os.ReadFile(data)
		if err != nil {
			return 0, err
		}
		var records []map[string]any
		if err := json.Unmarshal(content, &records); err != nil {
			return 0, err
		}
		var total int64
		for _, record := range records {
			total += int64(record["score"].(float64))
		}
		return total, nil
	case "directory":
		var count int64
		err := filepath.WalkDir(data, func(path string, entry os.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if path != data {
				count++
			}
			return nil
		})
		return count, err
	case "text":
		content, err := os.ReadFile(data)
		if err != nil {
			return 0, err
		}
		var count int64
		for _, line := range strings.Split(string(content), "\n") {
			if strings.Contains(line, "error") {
				count++
			}
		}
		return count, nil
	default:
		return 0, fmt.Errorf("unknown case: %s", name)
	}
}

func main() {
	if len(os.Args) < 2 || len(os.Args) > 3 {
		panic("expected case name and optional data path")
	}
	name, data := os.Args[1], ""
	if len(os.Args) == 3 {
		data = os.Args[2]
	}
	reader := bufio.NewScanner(os.Stdin)
	writer := bufio.NewWriter(os.Stdout)
	for reader.Scan() {
		if reader.Text() != "run" {
			panic("expected protocol command run")
		}
		var before, after runtime.MemStats
		runtime.ReadMemStats(&before)
		started := time.Now()
		checksum, err := run(name, data)
		elapsed := time.Since(started).Nanoseconds()
		runtime.ReadMemStats(&after)
		if err != nil {
			panic(err)
		}
		fmt.Fprintf(writer, "%d\t%d\t%d\t%d\n", checksum, elapsed, after.Mallocs-before.Mallocs, after.TotalAlloc-before.TotalAlloc)
		if err := writer.Flush(); err != nil {
			panic(err)
		}
	}
	if err := reader.Err(); err != nil {
		panic(err)
	}
}
