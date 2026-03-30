use sm_front::QuadVal;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDocument {
    pub fields: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry {
    pub key: String,
    pub value: ConfigValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigNumberKind {
    Integer,
    Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigNumber {
    pub raw: String,
    pub kind: ConfigNumberKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValue {
    Object(Vec<ConfigEntry>),
    String(String),
    Bool(bool),
    Quad(QuadVal),
    Number(ConfigNumber),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigParseError {
    pub pos: usize,
    pub message: String,
}

impl fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config parse error at {}: {}", self.pos, self.message)
    }
}

impl Error for ConfigParseError {}

pub fn parse_config_document(src: &str) -> Result<ConfigDocument, ConfigParseError> {
    let mut parser = ConfigParser::new(src);
    parser.skip_ws();
    if parser.peek() != Some(b'{') {
        return Err(parser.error("config document must start with '{'"));
    }
    let fields = parser.parse_object_entries()?;
    parser.skip_ws();
    if !parser.is_eof() {
        return Err(parser.error("unexpected trailing input after config document"));
    }
    Ok(ConfigDocument { fields })
}

struct ConfigParser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ConfigParser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn error(&self, message: impl Into<String>) -> ConfigParseError {
        ConfigParseError {
            pos: self.pos,
            message: message.into(),
        }
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if matches!(ch, b' ' | b'\n' | b'\r' | b'\t') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect_byte(&mut self, expected: u8, label: &str) -> Result<(), ConfigParseError> {
        match self.bump() {
            Some(ch) if ch == expected => Ok(()),
            Some(_) => Err(self.error(format!("expected {}", label))),
            None => Err(self.error(format!("expected {}", label))),
        }
    }

    fn parse_object_entries(&mut self) -> Result<Vec<ConfigEntry>, ConfigParseError> {
        self.expect_byte(b'{', "'{'")?;
        self.skip_ws();
        let mut entries = Vec::new();
        let mut seen = BTreeSet::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(entries);
        }

        loop {
            self.skip_ws();
            let key = self.parse_identifier()?;
            if !seen.insert(key.clone()) {
                return Err(self.error(format!("duplicate config key '{}'", key)));
            }
            self.skip_ws();
            self.expect_byte(b':', "':'")?;
            self.skip_ws();
            let value = self.parse_value()?;
            entries.push(ConfigEntry { key, value });
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_ws();
                    if self.peek() == Some(b'}') {
                        self.pos += 1;
                        break;
                    }
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.error("expected ',' or '}' after config field")),
            }
        }

        Ok(entries)
    }

    fn parse_value(&mut self) -> Result<ConfigValue, ConfigParseError> {
        match self.peek() {
            Some(b'{') => Ok(ConfigValue::Object(self.parse_object_entries()?)),
            Some(b'"') => Ok(ConfigValue::String(self.parse_string()?)),
            Some(b't') | Some(b'f') => Ok(ConfigValue::Bool(self.parse_bool()?)),
            Some(b'N' | b'F' | b'T' | b'S') => Ok(ConfigValue::Quad(self.parse_quad()?)),
            Some(b'-' | b'0'..=b'9') => Ok(ConfigValue::Number(self.parse_number()?)),
            Some(_) => Err(self.error("unsupported config value")),
            None => Err(self.error("expected config value")),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ConfigParseError> {
        let start = self.pos;
        match self.peek() {
            Some(ch) if is_ident_start(ch) => {
                self.pos += 1;
            }
            _ => return Err(self.error("expected identifier key")),
        }
        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(self.src[start..self.pos].to_string())
    }

    fn parse_string(&mut self) -> Result<String, ConfigParseError> {
        self.expect_byte(b'"', "'\"'")?;
        let mut out = String::new();
        loop {
            let ch = self.bump().ok_or_else(|| self.error("unterminated string"))?;
            match ch {
                b'"' => break,
                b'\\' => {
                    let escaped = self
                        .bump()
                        .ok_or_else(|| self.error("unterminated escape sequence"))?;
                    let mapped = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        _ => return Err(self.error("unsupported escape sequence")),
                    };
                    out.push(mapped);
                }
                _ => out.push(ch as char),
            }
        }
        Ok(out)
    }

    fn parse_bool(&mut self) -> Result<bool, ConfigParseError> {
        if self.try_keyword("true")? {
            return Ok(true);
        }
        if self.try_keyword("false")? {
            return Ok(false);
        }
        Err(self.error("expected 'true' or 'false'"))
    }

    fn parse_quad(&mut self) -> Result<QuadVal, ConfigParseError> {
        let ch = self.bump().ok_or_else(|| self.error("expected quad literal"))?;
        let quad = match ch {
            b'N' => QuadVal::N,
            b'F' => QuadVal::F,
            b'T' => QuadVal::T,
            b'S' => QuadVal::S,
            _ => return Err(self.error("expected quad literal")),
        };
        if matches!(self.peek(), Some(next) if is_ident_continue(next)) {
            return Err(self.error("quad literal must be delimited"));
        }
        Ok(quad)
    }

    fn parse_number(&mut self) -> Result<ConfigNumber, ConfigParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        let digits_start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.pos == digits_start {
            return Err(self.error("expected decimal digits"));
        }

        let kind = if self.peek() == Some(b'.') {
            self.pos += 1;
            let frac_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            if self.pos == frac_start {
                return Err(self.error("expected decimal digits after '.'"));
            }
            ConfigNumberKind::Decimal
        } else {
            ConfigNumberKind::Integer
        };

        if matches!(self.peek(), Some(next) if is_ident_continue(next)) {
            return Err(self.error("numeric literal must be delimited"));
        }

        Ok(ConfigNumber {
            raw: self.src[start..self.pos].to_string(),
            kind,
        })
    }

    fn try_keyword(&mut self, kw: &str) -> Result<bool, ConfigParseError> {
        let bytes = kw.as_bytes();
        if self.bytes.get(self.pos..self.pos + bytes.len()) != Some(bytes) {
            return Ok(false);
        }
        let end = self.pos + bytes.len();
        if matches!(self.bytes.get(end).copied(), Some(next) if is_ident_continue(next)) {
            return Err(self.error(format!("keyword '{}' must be delimited", kw)));
        }
        self.pos = end;
        Ok(true)
    }
}

fn is_ident_start(ch: u8) -> bool {
    matches!(ch, b'a'..=b'z' | b'A'..=b'Z' | b'_')
}

fn is_ident_continue(ch: u8) -> bool {
    is_ident_start(ch) || matches!(ch, b'0'..=b'9')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_document_accepts_nested_object_surface() {
        let doc = parse_config_document(
            r#"{
                enabled: true,
                mode: T,
                retries: 3,
                threshold: 0.25,
                nested: {
                    label: "alpha",
                },
            }"#,
        )
        .expect("config document should parse");

        assert_eq!(doc.fields.len(), 5);
        assert_eq!(doc.fields[0].key, "enabled");
        assert_eq!(doc.fields[0].value, ConfigValue::Bool(true));
        assert_eq!(doc.fields[1].value, ConfigValue::Quad(QuadVal::T));
        assert_eq!(
            doc.fields[2].value,
            ConfigValue::Number(ConfigNumber {
                raw: "3".to_string(),
                kind: ConfigNumberKind::Integer,
            })
        );
        assert_eq!(
            doc.fields[3].value,
            ConfigValue::Number(ConfigNumber {
                raw: "0.25".to_string(),
                kind: ConfigNumberKind::Decimal,
            })
        );
        let ConfigValue::Object(nested) = &doc.fields[4].value else {
            panic!("expected nested object");
        };
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].key, "label");
        assert_eq!(nested[0].value, ConfigValue::String("alpha".to_string()));
    }

    #[test]
    fn parse_config_document_rejects_duplicate_key_in_same_object() {
        let err = parse_config_document(
            r#"{
                enabled: true,
                enabled: false,
            }"#,
        )
        .expect_err("duplicate key must reject");

        assert!(err.message.contains("duplicate config key 'enabled'"));
    }

    #[test]
    fn parse_config_document_rejects_non_object_root() {
        let err = parse_config_document("true").expect_err("root must be object");
        assert!(err.message.contains("config document must start with '{'"));
    }
}
