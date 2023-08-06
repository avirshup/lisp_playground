use crate::Token::{ParenEnd, ParenStart};

fn main() {
    println!("Hello, world!");
}

/*************\
|* Tokenizer *|
\*************/
#[derive(Debug, Clone, PartialEq)]
struct Quote {
    sigil: String,
    mark: char,
    content: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    ParenStart,
    ParenEnd,
    Ident(String),
    StringLit(Quote),
}

fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut current_word: String = "".to_string();
    let mut current_quote: Option<Quote> = None;
    let mut is_escaped: bool = false;

    for char in s.chars() {
        /**********************************\
        |* Quoted string literal handling *|
        \**********************************/
        if let Some(mut quote) = current_quote.take() {
            // did we get a matching quote?
            if char == quote.mark {
                if is_escaped {
                    quote.content.push(char);
                    is_escaped = false;
                } else {
                    tokens.push(Token::StringLit(quote));
                    continue;
                }
            } else if is_escaped {
                // if not escaping a quote, treat it as literal slash
                quote.content.push('\\');
                quote.content.push(char);
                is_escaped = false;
            } else if char == '\\' {
                is_escaped = true;
            } else {
                quote.content.push(char);
            }
            current_quote = Some(quote);
        }
        // start a quote
        else if char == '\'' || char == '"' {
            current_quote = Some(Quote {
                sigil: current_word.clone(),
                mark: char,
                content: String::new(),
            });
            current_word.clear();
        }
        /**************************************\
        |* Everything besides string literals *|
        \**************************************/
        else if char == '(' || char == ')' || char.is_whitespace() {
            push_word(&mut tokens, &mut current_word);
            if char == '(' {
                tokens.push(ParenStart)
            } else if char == ')' {
                tokens.push(ParenEnd)
            };

        // continue with current identifier
        } else {
            current_word.push(char);
        }
    }
    push_word(&mut tokens, &mut current_word);

    tokens
}

#[inline]
fn push_word(tokens: &mut Vec<Token>, current_word: &mut String) {
    if !current_word.is_empty() {
        tokens.push(Token::Ident(current_word.clone()));
        current_word.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_quote() {
        assert_eq!(
            tokenize("(bee'hi')"),
            vec![
                ParenStart,
                Token::StringLit(Quote {
                    sigil: "bee".to_string(),
                    mark: '\'',
                    content: "hi".to_string()
                },),
                ParenEnd
            ]
        )
    }

    #[test]
    fn test_escaped_quote() {
        assert_eq!(
            tokenize("'hi\\'hi\\n'"),
            vec![Token::StringLit(Quote {
                sigil: String::new(),
                mark: '\'',
                content: "hi'hi\\n".to_string()
            })]
        )
    }

    #[test]
    fn test_tokenize_all_the_things() {
        assert_eq!(
            tokenize(")(hel\\lo ( 3.2 he(\"yo\\\"yo\"y"),
            vec![
                ParenEnd,
                ParenStart,
                Token::Ident("hel\\lo".to_string()),
                ParenStart,
                Token::Ident("3.2".to_string()),
                Token::Ident("he".to_string()),
                ParenStart,
                Token::StringLit(Quote {
                    sigil: String::new(),
                    mark: '"',
                    content: "yo\"yo".to_string()
                }),
                Token::Ident("y".to_string())
            ]
        )
    }
}
