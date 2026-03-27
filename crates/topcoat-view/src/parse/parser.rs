use crate::parse::{Node, NodeBuilder, NodeKind, Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
}

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    position: usize,
    builder: NodeBuilder<'a>,
    diagnostics: Vec<Diagnostic>,
}

pub fn parse<'a>(tokens: &'a [Token<'a>]) -> (Node<'a>, Vec<Diagnostic>) {
    let mut parser = Parser::new(tokens);
    let root = parser.parse_root();
    (root, parser.diagnostics)
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            tokens,
            position: 0,
            builder: NodeBuilder::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn parse_root(&mut self) -> Node<'a> {
        self.builder.begin(NodeKind::Root);
        self.skip_trivia();
        while !self.is_empty() {
            self.parse_element();
            self.skip_trivia();
        }
        self.builder.end();
        self.builder.finish()
    }

    fn parse_element(&mut self) {
        self.builder.begin(NodeKind::Element);
        if self.at(TokenKind::Ident) {
            self.bump();
        } else {
            self.error("expected element name");
        }
        self.builder.end();
    }

    fn is_empty(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn current(&self) -> Option<&'a Token<'a>> {
        self.tokens.get(self.position)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current().is_some_and(|t| t.kind() == kind)
    }

    /// Consume the current token and add it to the tree.
    fn bump(&mut self) -> Option<&'a Token<'a>> {
        let token = self.current()?;
        self.builder.token(token);
        self.position += 1;
        Some(token)
    }

    /// Consume a token of the expected kind, or emit an error.
    fn expect(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            let msg = format!("expected {:?}", kind);
            self.error(&msg);
            false
        }
    }

    /// Emit an error for the current token, wrapping it in an Error node
    /// so parsing can continue.
    fn error(&mut self, message: &str) {
        if let Some(token) = self.current() {
            let span = token.span();
            self.diagnostics.push(Diagnostic {
                message: message.to_string(),
                span,
            });
            // Wrap the unexpected token in an Error node and advance
            self.builder.begin(NodeKind::Error);
            self.bump();
            self.builder.end();
        } else {
            // Error at EOF
            let pos = self.tokens.last().map_or(0, |t| t.span().end());
            self.diagnostics.push(Diagnostic {
                message: message.to_string(),
                span: Span::new(pos, pos),
            });
        }
    }

    fn skip_trivia(&mut self) {
        while self.at(TokenKind::Whitespace) {
            self.bump();
        }
    }
}
