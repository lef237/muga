use std::{
    fmt::{self, Write},
    path::{Path, PathBuf},
};

use crate::span::Span;

pub const JSON_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub span: Span,
    pub related: DiagnosticList<RelatedNote>,
    pub suggestions: DiagnosticList<DiagnosticSuggestion>,
    pub context: DiagnosticList<DiagnosticContext>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::ops::Deref for DiagnosticCode {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl PartialEq<&str> for DiagnosticCode {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for DiagnosticCode {
    fn eq(&self, other: &String) -> bool {
        self.0 == other.as_str()
    }
}

impl PartialEq<&String> for DiagnosticCode {
    fn eq(&self, other: &&String) -> bool {
        self.0 == other.as_str()
    }
}

impl AsRef<str> for DiagnosticCode {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticList<T>(Box<[T]>);

impl<T> DiagnosticList<T> {
    pub fn new() -> Self {
        Self(Vec::new().into_boxed_slice())
    }

    pub fn push(&mut self, item: T) {
        let mut items = std::mem::take(&mut self.0).into_vec();
        items.push(item);
        self.0 = items.into_boxed_slice();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
}

impl<T> Default for DiagnosticList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::ops::Deref for DiagnosticList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> AsRef<[T]> for DiagnosticList<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a, T> IntoIterator for &'a DiagnosticList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedNote {
    pub message: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticSuggestion {
    pub message: String,
    pub span: Option<Span>,
    pub replacement: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticContext {
    Source {
        role: String,
        path: String,
        uri: String,
    },
    Package {
        role: String,
        path: String,
    },
    ArtifactRoot {
        role: String,
        path: String,
        uri: String,
    },
    ArtifactFile {
        role: String,
        artifact_kind: String,
        path: String,
        uri: String,
    },
    ArtifactHash {
        role: String,
        hash_kind: String,
        package_path: Option<String>,
        value: String,
    },
    RegenerationCommand {
        role: String,
        command: String,
    },
}

impl DiagnosticContext {
    pub fn source(
        role: impl Into<String>,
        path: impl Into<String>,
        uri: impl Into<String>,
    ) -> Self {
        Self::Source {
            role: role.into(),
            path: path.into(),
            uri: uri.into(),
        }
    }

    pub fn package(role: impl Into<String>, path: impl Into<String>) -> Self {
        Self::Package {
            role: role.into(),
            path: path.into(),
        }
    }

    pub fn artifact_root(
        role: impl Into<String>,
        path: impl Into<String>,
        uri: impl Into<String>,
    ) -> Self {
        Self::ArtifactRoot {
            role: role.into(),
            path: path.into(),
            uri: uri.into(),
        }
    }

    pub fn artifact_file(
        role: impl Into<String>,
        artifact_kind: impl Into<String>,
        path: impl Into<String>,
        uri: impl Into<String>,
    ) -> Self {
        Self::ArtifactFile {
            role: role.into(),
            artifact_kind: artifact_kind.into(),
            path: path.into(),
            uri: uri.into(),
        }
    }

    pub fn artifact_hash(
        role: impl Into<String>,
        hash_kind: impl Into<String>,
        package_path: Option<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::ArtifactHash {
            role: role.into(),
            hash_kind: hash_kind.into(),
            package_path,
            value: value.into(),
        }
    }

    pub fn regeneration_command(role: impl Into<String>, command: impl Into<String>) -> Self {
        Self::RegenerationCommand {
            role: role.into(),
            command: command.into(),
        }
    }
}

impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code: DiagnosticCode(code),
            message: message.into(),
            span,
            related: DiagnosticList::new(),
            suggestions: DiagnosticList::new(),
            context: DiagnosticList::new(),
        }
    }

    pub fn with_related(mut self, message: impl Into<String>, span: Span) -> Self {
        self.related.push(RelatedNote {
            message: message.into(),
            span,
        });
        self
    }

    pub fn with_suggestion(mut self, message: impl Into<String>) -> Self {
        self.suggestions.push(DiagnosticSuggestion {
            message: message.into(),
            span: None,
            replacement: None,
        });
        self
    }

    pub fn with_replacement(
        mut self,
        message: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        self.suggestions.push(DiagnosticSuggestion {
            message: message.into(),
            span: Some(span),
            replacement: Some(replacement.into()),
        });
        self
    }

    pub fn with_context(mut self, context: DiagnosticContext) -> Self {
        self.add_context(context);
        self
    }

    pub fn add_context(&mut self, context: DiagnosticContext) {
        self.context.push(context);
    }

    pub fn to_json_object(&self) -> String {
        self.to_json_object_with_context(&[])
    }

    pub fn to_json_object_with_context(&self, context: &[DiagnosticContext]) -> String {
        let mut output = String::new();
        output.push('{');
        output.push_str("\"code\":");
        push_json_string(&mut output, self.code.as_str());
        output.push_str(",\"severity\":\"error\"");
        output.push_str(",\"message\":");
        push_json_string(&mut output, &self.message);
        output.push_str(",\"span\":");
        push_span_json(&mut output, self.span);
        output.push_str(",\"related\":[");
        for (index, note) in self.related.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push('{');
            output.push_str("\"message\":");
            push_json_string(&mut output, &note.message);
            output.push_str(",\"span\":");
            push_span_json(&mut output, note.span);
            output.push('}');
        }
        output.push_str("],\"suggestions\":[");
        for (index, suggestion) in self.suggestions.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push('{');
            output.push_str("\"message\":");
            push_json_string(&mut output, &suggestion.message);
            output.push_str(",\"span\":");
            if let Some(span) = suggestion.span {
                push_span_json(&mut output, span);
            } else {
                output.push_str("null");
            }
            output.push_str(",\"replacement\":");
            if let Some(replacement) = &suggestion.replacement {
                push_json_string(&mut output, replacement);
            } else {
                output.push_str("null");
            }
            output.push('}');
        }
        output.push_str("],\"context\":");
        push_context_json_array(&mut output, context, self.context.as_slice());
        output.push('}');
        output
    }
}

pub fn diagnostics_json_array(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();
    output.push('[');
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&diagnostic.to_json_object());
    }
    output.push(']');
    output
}

pub fn diagnostics_json_array_with_context(
    diagnostics: &[Diagnostic],
    context: &[DiagnosticContext],
) -> String {
    let mut output = String::new();
    output.push('[');
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&diagnostic.to_json_object_with_context(context));
    }
    output.push(']');
    output
}

fn push_context_json_array(
    output: &mut String,
    command_context: &[DiagnosticContext],
    diagnostic_context: &[DiagnosticContext],
) {
    output.push('[');
    let mut needs_comma = false;
    for context in command_context.iter().chain(diagnostic_context.iter()) {
        if needs_comma {
            output.push(',');
        } else {
            needs_comma = true;
        }
        push_context_json(output, context);
    }
    output.push(']');
}

fn push_context_json(output: &mut String, context: &DiagnosticContext) {
    match context {
        DiagnosticContext::Source { role, path, uri } => {
            output.push_str("{\"kind\":\"source\",\"role\":");
            push_json_string(output, role);
            output.push_str(",\"path\":");
            push_json_string(output, path);
            output.push_str(",\"uri\":");
            push_json_string(output, uri);
            output.push('}');
        }
        DiagnosticContext::Package { role, path } => {
            output.push_str("{\"kind\":\"package\",\"role\":");
            push_json_string(output, role);
            output.push_str(",\"path\":");
            push_json_string(output, path);
            output.push('}');
        }
        DiagnosticContext::ArtifactRoot { role, path, uri } => {
            output.push_str("{\"kind\":\"artifactRoot\",\"role\":");
            push_json_string(output, role);
            output.push_str(",\"path\":");
            push_json_string(output, path);
            output.push_str(",\"uri\":");
            push_json_string(output, uri);
            output.push('}');
        }
        DiagnosticContext::ArtifactFile {
            role,
            artifact_kind,
            path,
            uri,
        } => {
            output.push_str("{\"kind\":\"artifactFile\",\"role\":");
            push_json_string(output, role);
            output.push_str(",\"artifactKind\":");
            push_json_string(output, artifact_kind);
            output.push_str(",\"path\":");
            push_json_string(output, path);
            output.push_str(",\"uri\":");
            push_json_string(output, uri);
            output.push('}');
        }
        DiagnosticContext::ArtifactHash {
            role,
            hash_kind,
            package_path,
            value,
        } => {
            output.push_str("{\"kind\":\"artifactHash\",\"role\":");
            push_json_string(output, role);
            output.push_str(",\"hashKind\":");
            push_json_string(output, hash_kind);
            if let Some(package_path) = package_path {
                output.push_str(",\"packagePath\":");
                push_json_string(output, package_path);
            }
            output.push_str(",\"value\":");
            push_json_string(output, value);
            output.push('}');
        }
        DiagnosticContext::RegenerationCommand { role, command } => {
            output.push_str("{\"kind\":\"regenerationCommand\",\"role\":");
            push_json_string(output, role);
            output.push_str(",\"command\":");
            push_json_string(output, command);
            output.push('}');
        }
    }
}

pub fn artifact_file_context(
    role: impl Into<String>,
    artifact_kind: impl Into<String>,
    path: &Path,
) -> DiagnosticContext {
    DiagnosticContext::artifact_file(
        role,
        artifact_kind,
        path.display().to_string(),
        file_uri_for_path(path),
    )
}

pub fn artifact_hash_context(
    role: impl Into<String>,
    hash_kind: impl Into<String>,
    package_path: Option<&str>,
    value: impl Into<String>,
) -> DiagnosticContext {
    DiagnosticContext::artifact_hash(role, hash_kind, package_path.map(str::to_string), value)
}

pub fn regeneration_command_context(
    role: impl Into<String>,
    command: impl Into<String>,
) -> DiagnosticContext {
    DiagnosticContext::regeneration_command(role, command)
}

pub fn file_uri_for_path(path: &Path) -> String {
    let path = absolute_path_for_uri(path);
    let path_text = path.to_string_lossy();
    let mut uri = String::from("file://");
    if !path_text.starts_with('/') {
        uri.push('/');
    }
    uri.push_str(&percent_encode_uri_path(&path_text));
    uri
}

fn absolute_path_for_uri(path: &Path) -> PathBuf {
    if let Ok(canonical_path) = path.canonicalize() {
        return canonical_path;
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn percent_encode_uri_path(path: &str) -> String {
    let mut output = String::new();
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' | b':' => {
                output.push(byte as char)
            }
            _ => write!(&mut output, "%{byte:02X}")
                .expect("writing URI escape to String should not fail"),
        }
    }
    output
}

fn push_span_json(output: &mut String, span: Span) {
    write!(
        output,
        "{{\"start\":{{\"line\":{},\"column\":{}}},\"end\":{{\"line\":{},\"column\":{}}}}}",
        span.start.line, span.start.column, span.end.line, span.end.column
    )
    .expect("writing JSON to String should not fail");
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => {
                write!(output, "\\u{:04x}", ch as u32)
                    .expect("writing JSON escape to String should not fail");
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} {}",
            self.span.start.line, self.span.start.column, self.code, self.message
        )?;
        for note in &self.related {
            write!(
                f,
                "\n  note: {}:{}: {}",
                note.span.start.line, note.span.start.column, note.message
            )?;
        }
        for suggestion in &self.suggestions {
            match (suggestion.span, suggestion.replacement.as_ref()) {
                (Some(span), Some(replacement)) => write!(
                    f,
                    "\n  help: {}:{}: {}; replace with `{}`",
                    span.start.line, span.start.column, suggestion.message, replacement
                )?,
                (Some(span), None) => write!(
                    f,
                    "\n  help: {}:{}: {}",
                    span.start.line, span.start.column, suggestion.message
                )?,
                (None, Some(replacement)) => {
                    write!(f, "\n  help: {}; use `{}`", suggestion.message, replacement)?
                }
                (None, None) => write!(f, "\n  help: {}", suggestion.message)?,
            }
        }
        Ok(())
    }
}
