use crate::parser::Parser;
use crate::token::Token;

impl<'a> Parser<'a> {
    fn is_conditional_head(&mut self) -> bool {
        match self.peek_unexpanded_token() {
            Some(Token::ControlSequence(cs)) => {
                cs == "else" || cs == "fi" || cs == "iftrue" || cs == "iffalse"
            }
            _ => false,
        }
    }

    // Skips tokens until a \fi or \else is parsed. Returns true if the token
    // we found is \else, false if it is \fi.
    fn skip_to_fi_or_else(&mut self) -> bool {
        let mut ends_with_else = false;
        loop {
            match self.lex_unexpanded_token().unwrap() {
                Token::ControlSequence(ref cs) if cs == "fi" => break,
                Token::ControlSequence(ref cs) if cs == "else" => {
                    ends_with_else = true;
                    break;
                }
                _ => (),
            }
        }
        ends_with_else
    }

    fn skip_from_else(&mut self) {
        // When we encounter an \else, we know that we're in a 'true'
        // conditional because in a 'false' conditional, we always already
        // parse the \else token in skip_to_fi_or_else(). Thus, we just need to
        // skip tokens until we see a \fi.
        loop {
            let token = self.lex_unexpanded_token().unwrap();
            if token == Token::ControlSequence("fi".to_string()) {
                break;
            }
        }
    }

    fn handle_true(&mut self) {
        self.conditional_depth += 1;
    }

    fn handle_false(&mut self) {
        if self.skip_to_fi_or_else() {
            // If we skipped all the way to a \fi, we don't add to our depth of
            // conditionals because we already exited this one. If we only
            // skipped to a \else, we are now inside a conditional.
            self.conditional_depth += 1;
        }
    }

    fn expand_conditional(&mut self) {
        match self.lex_unexpanded_token() {
            Some(Token::ControlSequence(ref cs)) if cs == "fi" => {
                if self.conditional_depth == 0 {
                    panic!("Extra \\fi");
                }
                self.conditional_depth -= 1;
            }
            Some(Token::ControlSequence(ref cs)) if cs == "else" => {
                if self.conditional_depth == 0 {
                    panic!("Extra \\else");
                }
                self.conditional_depth -= 1;
                self.skip_from_else();
            }
            Some(Token::ControlSequence(ref cs)) if cs == "iftrue" => self.handle_true(),
            Some(Token::ControlSequence(ref cs)) if cs == "iffalse" => self.handle_false(),
            _ => panic!("unimplemented"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::rc::Rc;

    use crate::category::Category;
    use crate::makro::{Macro, MacroListElem};
    use crate::state::TeXState;

    #[test]
    fn it_parses_single_body_iftrue() {
        let state = TeXState::new();
        let mut parser = Parser::new(&["\\iftrue x\\fi%"], &state);

        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(
            parser.lex_unexpanded_token(),
            Some(Token::Char('x', Category::Letter))
        );
        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(parser.lex_unexpanded_token(), None);
    }

    #[test]
    fn it_parses_iftrue_with_else() {
        let state = TeXState::new();
        let mut parser = Parser::new(&["\\iftrue x\\else y\\fi%"], &state);

        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(
            parser.lex_unexpanded_token(),
            Some(Token::Char('x', Category::Letter))
        );
        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(parser.lex_unexpanded_token(), None);
    }

    #[test]
    fn it_parses_single_body_iffalse() {
        let state = TeXState::new();
        let mut parser = Parser::new(&["\\iffalse x\\fi%"], &state);

        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(parser.lex_unexpanded_token(), None);
    }

    #[test]
    fn it_parses_iffalse_with_else() {
        let state = TeXState::new();
        let mut parser = Parser::new(&["\\iffalse x\\else y\\fi%"], &state);

        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(
            parser.lex_unexpanded_token(),
            Some(Token::Char('y', Category::Letter))
        );
        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(parser.lex_unexpanded_token(), None);
    }

    #[test]
    fn it_expand_macros_in_true_bodies_but_not_false_bodies() {
        let state = TeXState::new();
        state.set_macro(
            Token::ControlSequence("a".to_string()),
            Rc::new(Macro::new(
                vec![],
                vec![
                    MacroListElem::Token(Token::Char('x', Category::Letter)),
                    MacroListElem::Token(Token::ControlSequence("else".to_string())),
                    MacroListElem::Token(Token::Char('y', Category::Letter)),
                ],
            )),
        );
        state.set_macro(
            Token::ControlSequence("b".to_string()),
            Rc::new(Macro::new(
                vec![],
                vec![
                    MacroListElem::Token(Token::Char('z', Category::Letter)),
                    MacroListElem::Token(Token::ControlSequence("fi".to_string())),
                ],
            )),
        );
        let mut parser = Parser::new(&["\\iftrue w\\a\\b\\fi%"], &state);

        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(
            parser.lex_expanded_token(),
            Some(Token::Char('w', Category::Letter))
        );
        assert_eq!(
            parser.lex_expanded_token(),
            Some(Token::Char('x', Category::Letter))
        );
        assert_eq!(parser.is_conditional_head(), true);
        parser.expand_conditional();
        assert_eq!(parser.lex_unexpanded_token(), None);
    }
}
