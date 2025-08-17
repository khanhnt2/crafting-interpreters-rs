use crate::{
    error::ParsingError,
    expr::{
        AssignExpr, BinaryExpr, CallExpr, Expr, GetExpr, GroupingExpr, LambdaExpr, LiteralExpr,
        LogicalExpr, SetExpr, SuperExpr, TernaryExpr, ThisExpr, UnaryExpr, VariableExpr,
    },
    function::FunctionType,
    object::Object,
    stmt::{
        BlockStmt, ClassStmt, ExpressionStmt, FunctionStmt, IfStmt, PrintStmt, ReturnStmt, Stmt,
        VarStmt, WhileStmt,
    },
    token::{Token, TokenIdentity, TokenValue},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        // We eliminate comments from the token stream
        let tokens = tokens
            .clone()
            .extract_if(.., |token| token.id != TokenIdentity::Comment)
            .collect();
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParsingError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration(false)?);
        }
        Ok(statements)
    }

    fn declaration(&mut self, in_loop: bool) -> Result<Stmt, ParsingError> {
        if self.match_token(vec![TokenIdentity::Class]) {
            self.class_declaration().map(Stmt::Class)
        } else if self.match_token(vec![TokenIdentity::Fun])
            && self.check(TokenIdentity::Identifier)
        {
            self.function(FunctionType::Function).map(Stmt::Function)
        } else if self.match_token(vec![TokenIdentity::Var]) {
            self.var_declaration().map(Stmt::Var)
        } else {
            self.statement(in_loop)
        }
    }

    fn class_declaration(&mut self) -> Result<ClassStmt, ParsingError> {
        let name = self
            .consume(TokenIdentity::Identifier, "Expect class name.")?
            .to_owned();
        let superclass = if self.match_token(vec![TokenIdentity::Less]) {
            self.consume(TokenIdentity::Identifier, "Expect superclass name.")?;
            Some(VariableExpr::new(self.previous().to_owned()))
        } else {
            None
        };

        let mut methods = Vec::new();
        let mut static_methods = Vec::new();
        let mut getter_methods = Vec::new();

        self.consume(TokenIdentity::LeftBrace, "Expect '{' before class body.")?;
        while !self.check(TokenIdentity::RightBrace) && !self.is_at_end() {
            if self.match_token(vec![TokenIdentity::Class]) {
                static_methods.push(self.function(FunctionType::StaticMethod)?);
            } else {
                let method = self.function(FunctionType::Method)?;
                if method.kind == FunctionType::GetterMethod {
                    getter_methods.push(method);
                } else {
                    methods.push(method);
                }
            }
        }
        self.consume(TokenIdentity::RightBrace, "Expect '}' after class body.")?;

        Ok(ClassStmt::new(
            name,
            superclass,
            methods,
            static_methods,
            getter_methods,
        ))
    }

    fn var_declaration(&mut self) -> Result<VarStmt, ParsingError> {
        let name = self
            .consume(TokenIdentity::Identifier, "Expect variable name.")?
            .to_owned();
        let initializer = if self.match_token(vec![TokenIdentity::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenIdentity::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(VarStmt::new(name, initializer))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenIdentity::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(
            TokenIdentity::RightParen,
            "Expect ')' after while condition.",
        )?;

        self.consume(TokenIdentity::LeftBrace, "Expect '{' before while body.")?;
        let body = self.block(true)?;

        Ok(Stmt::While(WhileStmt::new(condition, body)))
    }

    fn statement(&mut self, in_loop: bool) -> Result<Stmt, ParsingError> {
        if self.match_token(vec![TokenIdentity::For]) {
            self.for_statement()
        } else if self.match_token(vec![TokenIdentity::Print]) {
            self.print_statement()
        } else if self.match_token(vec![TokenIdentity::Return]) {
            self.return_statement()
        } else if self.match_token(vec![TokenIdentity::While]) {
            self.while_statement()
        } else if self.match_token(vec![TokenIdentity::If]) {
            self.if_statement(in_loop)
        } else if self.match_token(vec![TokenIdentity::LeftBrace]) {
            Ok(Stmt::Block(self.block(in_loop)?))
        } else if self.match_token(vec![TokenIdentity::Break]) {
            if !in_loop {
                return Err(ParsingError::new(
                    self.previous().to_owned(),
                    "Can only use 'break' inside loops.",
                ));
            }
            self.break_statement()
        } else if self.match_token(vec![TokenIdentity::Continue]) {
            if !in_loop {
                return Err(ParsingError::new(
                    self.previous().to_owned(),
                    "Can only use 'continue' inside loops.",
                ));
            }
            self.continue_statement()
        } else {
            self.expression_statement()
        }
    }

    fn break_statement(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenIdentity::Semicolon, "Expect ';' after break.")?;
        Ok(Stmt::Break)
    }

    fn continue_statement(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenIdentity::Semicolon, "Expect ';' after continue.")?;
        Ok(Stmt::Continue)
    }

    fn for_statement(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenIdentity::LeftParen, "Expect '(' after 'for'.")?;
        let initializer = if self.match_token(vec![TokenIdentity::Semicolon]) {
            None
        } else if self.match_token(vec![TokenIdentity::Var]) {
            Some(Stmt::Var(self.var_declaration()?))
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.match_token(vec![TokenIdentity::Semicolon]) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenIdentity::Semicolon, "Expect ';' after for condition.")?;

        let increment = if self.match_token(vec![TokenIdentity::RightParen]) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenIdentity::RightParen, "Expect ')' after for clauses.")?;

        self.consume(TokenIdentity::LeftBrace, "Expect '{' before for body.")?;
        let mut body = self.block(true)?;

        if let Some(increment) = increment {
            body.statements
                .push(Stmt::Expression(ExpressionStmt::new(increment)));
        }

        let condition = condition.unwrap_or(Expr::Literal(LiteralExpr::new(Object::Boolean(true))));
        let mut stmt = Stmt::While(WhileStmt::new(condition, body));

        if let Some(initializer) = initializer {
            stmt = Stmt::Block(BlockStmt::new(vec![initializer, stmt]));
        }
        Ok(stmt)
    }

    fn if_statement(&mut self, in_loop: bool) -> Result<Stmt, ParsingError> {
        self.consume(TokenIdentity::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenIdentity::RightParen, "Expect ')' after if condition.")?;

        self.consume(TokenIdentity::LeftBrace, "Expect '{' before if body.")?;
        let then_branch = self.block(in_loop)?;
        let else_branch = if self.match_token(vec![TokenIdentity::Else]) {
            self.consume(TokenIdentity::LeftBrace, "Expect '{' before else body.")?;
            Some(self.block(in_loop)?)
        } else {
            None
        };
        Ok(Stmt::If(IfStmt::new(condition, then_branch, else_branch)))
    }

    fn print_statement(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenIdentity::LeftParen, "Expect '(' after 'print'.")?;
        let value = self.expression()?;
        self.consume(TokenIdentity::RightParen, "Expect ')' after arguments.")?;
        self.consume(
            TokenIdentity::Semicolon,
            "Expect ';' after print statement.",
        )?;
        Ok(Stmt::Print(PrintStmt::new(value)))
    }

    fn return_statement(&mut self) -> Result<Stmt, ParsingError> {
        let keyword = self.previous().to_owned();
        let value = if !self.check(TokenIdentity::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenIdentity::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(ReturnStmt::new(keyword, value)))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParsingError> {
        let expression = self.expression()?;

        // The semicolon isn't at the end of lambda expression.
        if let Expr::Lambda(_) = expression {
        } else {
            self.consume(TokenIdentity::Semicolon, "Expect ';' after expression.")?;
        }

        Ok(Stmt::Expression(ExpressionStmt::new(expression)))
    }

    fn function(&mut self, mut kind: FunctionType) -> Result<FunctionStmt, ParsingError> {
        let name = self
            .consume(TokenIdentity::Identifier, &format!("Expect {kind} name."))?
            .to_owned();
        let mut parameters = Vec::new();
        if kind == FunctionType::Method && self.check(TokenIdentity::LeftBrace) {
            // Getter methods don't have parameters.
            kind = FunctionType::GetterMethod;
        } else {
            if name.value == TokenValue::String("init".to_string()) {
                kind = FunctionType::Initializer;
            }
            self.consume(
                TokenIdentity::LeftParen,
                &format!("Expect '(' after {kind} name."),
            )?;
            if !self.check(TokenIdentity::RightParen) {
                loop {
                    if parameters.len() >= 255 {
                        return Err(ParsingError::new(
                            self.peek().to_owned(),
                            "Can't have more than 255 parameters.",
                        ));
                    }
                    parameters.push(
                        self.consume(TokenIdentity::Identifier, "Expect parameter name.")?
                            .to_owned(),
                    );

                    if !self.match_token(vec![TokenIdentity::Comma]) {
                        break;
                    }
                }
            }
            self.consume(TokenIdentity::RightParen, "Expect ')' after parameters.")?;
        }

        self.consume(
            TokenIdentity::LeftBrace,
            &format!("Expect '{{' before {kind} body."),
        )?;
        let body = self.block(false)?;

        Ok(FunctionStmt::new(name.to_owned(), parameters, body, kind))
    }

    fn block(&mut self, in_loop: bool) -> Result<BlockStmt, ParsingError> {
        if self.previous().id != TokenIdentity::LeftBrace {
            return Err(ParsingError::new(
                self.previous().to_owned(),
                "Expect '{' before block.",
            ));
        }

        let mut statements = Vec::new();
        while !self.check(TokenIdentity::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration(in_loop)?);
        }
        self.consume(TokenIdentity::RightBrace, "Expect '}' after block.")?;
        // self.consume(TokenIdentity::Semicolon, "Expect ';' after block.")?;

        Ok(BlockStmt::new(statements))
    }

    fn expression(&mut self) -> Result<Expr, ParsingError> {
        self.lambda()
    }

    fn lambda(&mut self) -> Result<Expr, ParsingError> {
        if self.previous().id == TokenIdentity::Fun || self.match_token(vec![TokenIdentity::Fun]) {
            self.consume(
                TokenIdentity::LeftParen,
                "Expect '(' after 'fun' for lambda.",
            )?;
            let mut parameters = Vec::new();
            if !self.check(TokenIdentity::RightParen) {
                loop {
                    if parameters.len() >= 255 {
                        return Err(ParsingError::new(
                            self.peek().to_owned(),
                            "Can't have more than 255 parameters.",
                        ));
                    }
                    parameters.push(
                        self.consume(TokenIdentity::Identifier, "Expect parameter name.")?
                            .to_owned(),
                    );

                    if !self.match_token(vec![TokenIdentity::Comma]) {
                        break;
                    }
                }
            }
            self.consume(TokenIdentity::RightParen, "Expect ')' after parameters.")?;

            self.consume(TokenIdentity::LeftBrace, "Expect '{' before function body.")?;
            let body = self.block(false)?;

            Ok(Expr::Lambda(Box::new(LambdaExpr::new(parameters, body))))
        } else {
            self.ternary()
        }
    }

    fn ternary(&mut self) -> Result<Expr, ParsingError> {
        let expression = self.assignment()?;

        if self.match_token(vec![TokenIdentity::Question]) {
            let then_branch = self.expression()?;
            self.consume(TokenIdentity::Colon, "Expect ':' after then branch.")?;
            let else_branch = self.expression()?;
            Ok(Expr::Ternary(Box::new(TernaryExpr::new(
                expression,
                then_branch,
                else_branch,
            ))))
        } else {
            Ok(expression)
        }
    }

    fn assignment(&mut self) -> Result<Expr, ParsingError> {
        let expr = self.or()?;

        if self.match_token(vec![TokenIdentity::Equal]) {
            let equals = self.previous().to_owned();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(var) => Ok(Expr::Assign(Box::new(AssignExpr::new(var.name, value)))),
                Expr::Get(get) => Ok(Expr::Set(Box::new(SetExpr::new(
                    get.object, get.name, value,
                )))),
                _ => Err(ParsingError::new(equals, "Invalid assignment target.")),
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.and()?;

        while self.match_token(vec![TokenIdentity::Or]) {
            let operator = self.previous().to_owned();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(LogicalExpr::new(expr, operator, right)));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.equality()?;

        while self.match_token(vec![TokenIdentity::And]) {
            let operator = self.previous().to_owned();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(LogicalExpr::new(expr, operator, right)));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![TokenIdentity::BangEqual, TokenIdentity::EqualEqual]) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(BinaryExpr::new(expr, operator, right)));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.term()?;

        while self.match_token(vec![
            TokenIdentity::Greater,
            TokenIdentity::GreaterEqual,
            TokenIdentity::Less,
            TokenIdentity::LessEqual,
        ]) {
            let operator = self.previous().to_owned();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(BinaryExpr::new(expr, operator, right)));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.factor()?;

        while self.match_token(vec![TokenIdentity::Minus, TokenIdentity::Plus]) {
            let operator = self.previous().to_owned();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(BinaryExpr::new(expr, operator, right)));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.unary()?;

        while self.match_token(vec![TokenIdentity::Slash, TokenIdentity::Star]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(BinaryExpr::new(expr, operator, right)));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParsingError> {
        if self.match_token(vec![TokenIdentity::Bang, TokenIdentity::Minus]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            Ok(Expr::Unary(Box::new(UnaryExpr::new(operator, right))))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(vec![TokenIdentity::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(vec![TokenIdentity::Dot]) {
                let name =
                    self.consume(TokenIdentity::Identifier, "Expect property name after '.'.")?;
                expr = Expr::Get(Box::new(GetExpr::new(expr, name.to_owned())));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParsingError> {
        let mut arguments = Vec::new();

        if !self.check(TokenIdentity::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(ParsingError::new(
                        self.peek().to_owned(),
                        "Can't have more than 255 arguments.",
                    ));
                }
                arguments.push(self.expression()?);
                if !self.match_token(vec![TokenIdentity::Comma]) {
                    break;
                }
            }
        }

        let paren = self
            .consume(TokenIdentity::RightParen, "Expect ')' after arguments.")?
            .to_owned();

        Ok(Expr::Call(Box::new(CallExpr::new(
            callee, paren, arguments,
        ))))
    }

    fn primary(&mut self) -> Result<Expr, ParsingError> {
        let token_type = self.advance().id;
        match token_type {
            TokenIdentity::False => Ok(Expr::Literal(LiteralExpr::new(Object::Boolean(false)))),
            TokenIdentity::True => Ok(Expr::Literal(LiteralExpr::new(Object::Boolean(true)))),
            TokenIdentity::Nil => Ok(Expr::Literal(LiteralExpr::new(Object::Nil))),
            TokenIdentity::Number => match self.previous().value {
                TokenValue::Number(num) => Ok(Expr::Literal(LiteralExpr::new(Object::Number(num)))),
                _ => panic!("Unexpected object type"),
            },
            TokenIdentity::String => match self.previous().value.clone() {
                TokenValue::String(s) => Ok(Expr::Literal(LiteralExpr::new(Object::String(s)))),
                _ => panic!("Unexpected object type"),
            },
            TokenIdentity::Super => {
                let keyword = self.previous().to_owned();
                self.consume(TokenIdentity::Dot, "Expect '.' after 'super'.")?;
                let method =
                    self.consume(TokenIdentity::Identifier, "Expect superclass method name.")?;
                Ok(Expr::Super(SuperExpr::new(keyword, method.to_owned())))
            }
            TokenIdentity::This => Ok(Expr::This(ThisExpr::new(self.previous().to_owned()))),
            TokenIdentity::Identifier => Ok(Expr::Variable(VariableExpr::new(
                self.previous().to_owned(),
            ))),
            TokenIdentity::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenIdentity::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(GroupingExpr::new(expr))))
            }
            _ => Err(ParsingError::new(
                self.peek().to_owned(),
                "Unexpected expression",
            )),
        }
    }

    fn consume(&mut self, id: TokenIdentity, message: &str) -> Result<&Token, ParsingError> {
        if self.check(id) {
            return Ok(self.advance());
        }

        Err(ParsingError::new(self.peek().to_owned(), message))
    }

    fn match_token(&mut self, ids: Vec<TokenIdentity>) -> bool {
        for id in ids {
            if self.check(id) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, id: TokenIdentity) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().id == id
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().id == TokenIdentity::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
