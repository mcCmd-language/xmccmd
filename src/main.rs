use std::{
    fs::read_to_string,
    env::args, collections::HashMap, iter::zip
};

#[derive(PartialEq, Clone, Debug)]
enum TokenType {
    None,
    TokenScopeStart,
    TokenScopeEnd,
    TokenIdentifier,
    TokenVarName,
    TokenVarOptionKey,
    TokenVarOptionValue,
    TokenVarOptionSep,
    TokenFunStart,
    TokenFunName,
    TokenFunArgKey,
    TokenFunArgValue,
    TokenFunCall,
    TokenFunCallArg
}

#[derive(Debug)]
enum Type {
    String, Int, Float, Bool
}

#[derive(Debug)]
enum Value {
    String(String),
    Int(isize),
    Float(f64),
    Bool(bool)
}

#[derive(Debug)]
enum LexType {
    Function(String, Vec<ArgumentDef>, Vec<LexType>),
    Variable(String, Type, Value),
    FunctionCall(String, Vec<Value>)
}

#[derive(Debug)]
#[allow(dead_code)]
struct ArgumentDef {
    name: String,
    argtype: Type
}

fn compare_token_type(tokens: &Vec<Token>, token_type: TokenType) -> bool {
    if tokens.is_empty() {
        return false;
    }
    tokens[tokens.len()-1].token_type == token_type
}

#[derive(Clone, Debug)]
struct Token {
    token_type: TokenType,
    value: String
}

#[derive(Debug, Clone)]
struct Function<'a> {
    args: &'a Vec<ArgumentDef>,
    scope: &'a Vec<LexType>
}

fn tokenizer(filecontent: Vec<String>) -> Vec<Token> {
    let mut buffer = String::new();
    let mut tokens: Vec<Token> = Vec::new();
    let mut i: usize = 0;
    while filecontent.get(i).is_some() {
        let buffer_without = buffer.strip_prefix("   ").unwrap_or(&buffer).strip_prefix(' ').unwrap_or(&buffer);
        let mut token_found = false;
        let mut token_semi_found = false;
        let mut ignored = false;
        let t = &filecontent[i];
        let mut token = Token{token_type: TokenType::None, value: ";".to_string()};
        if (tokens.is_empty() || compare_token_type(&tokens, TokenType::None) || compare_token_type(&tokens, TokenType::TokenScopeStart)) && buffer_without == "@ " {
            token_found = true;
            token.token_type = TokenType::TokenIdentifier;
            token.value = buffer.clone();
        } else if compare_token_type(&tokens, TokenType::TokenIdentifier) && buffer.ends_with('[') {
            token_found = true;
            token.token_type = TokenType::TokenVarName;
            token.value = buffer[0..buffer.len()-1].to_string()
        } else if compare_token_type(&tokens, TokenType::TokenIdentifier) && buffer == "/" {
            token_found = true;
            token.token_type = TokenType::TokenFunStart;
            token.value = buffer.clone();
        } else if compare_token_type(&tokens, TokenType::TokenFunStart) && buffer.ends_with(' ') {
            token_found = true;
            token.token_type = TokenType::TokenFunName;
            token.value = buffer[0..buffer.len()-1].to_string()
        } else if compare_token_type(&tokens, TokenType::TokenFunName) && t == "#" {
            let args = buffer.split(' ').map(|x| x.to_string()).collect::<Vec<String>>();
            println!("buffers: {buffer} {:#?}", args);
            for j in args {
                if !j.is_empty() {
                    let arg2 = j.split('@').map(|x| x.to_string()).collect::<Vec<String>>();
                    tokens.push(Token {
                        token_type: TokenType::TokenFunArgKey,
                        value: arg2[0].clone()
                    });
                    tokens.push(Token {
                        token_type: TokenType::TokenFunArgValue,
                        value: arg2[1].clone()
                    });
                }
            }
            buffer = String::new();
            while &filecontent[i] != "#" {
                i += 1;
            }
            tokens.push(Token {
                token_type: TokenType::TokenScopeStart,
                value: "#".to_string()
            });
            ignored = true;
        }
        else if (compare_token_type(&tokens, TokenType::TokenVarName) || compare_token_type(&tokens, TokenType::TokenVarOptionSep)) && buffer.ends_with('=') {
            token_found = true;
            token.token_type = TokenType::TokenVarOptionKey;
            token.value = buffer[0..buffer.len()-1].to_string()
        } else if compare_token_type(&tokens, TokenType::TokenVarOptionKey) && (buffer.ends_with(']') || buffer.ends_with(',')) {
            token_found = true;
            token.token_type = TokenType::TokenVarOptionValue;
            token.value = buffer[0..buffer.len()-1].to_string();
            if buffer.ends_with(',') {
                tokens.push(token.clone());
                token.token_type = TokenType::TokenVarOptionSep;
                token.value = ",".to_string();
            }
        } else if compare_token_type(&tokens, TokenType::None) && buffer_without.starts_with('/') && buffer_without.ends_with(' ') {
            token_found = true;
            token.token_type = TokenType::TokenFunCall;
            token.value = buffer_without[0..buffer_without.len()-1].to_string();
        } else if compare_token_type(&tokens, TokenType::TokenFunCall) && t == ";" {
            let args = buffer.split(' ').map(|x| x.to_string()).collect::<Vec<String>>();
            for j in args {
                tokens.push(Token {
                    token_type: TokenType::TokenFunCallArg,
                    value: j.strip_prefix('"').unwrap_or(&j).strip_suffix('"').unwrap_or(&j).to_string()
                });
            }
            buffer = String::new();
        } else if compare_token_type(&tokens, TokenType::None) && buffer == "#" {
            token.token_type = TokenType::TokenScopeEnd;
            token.value = "#".to_string();
            tokens.push(token.clone());
            ignored = true;
        }
        if !compare_token_type(&tokens, TokenType::None) && t == ";" {
            token_semi_found = true;
        }
        if token_found {
            tokens.push(token);
            buffer = String::new();
        }
        if t != "\n" && !ignored {
            buffer.push_str(t.as_str());
        }
        if token_semi_found {
            let atoken = Token{token_type: TokenType::None, value: ";".to_string()};
            tokens.push(atoken);
            buffer = String::new();
        }
        i += 1;
    }
    tokens
}

fn parse(tokens: &[Token]) -> Vec<LexType> {
    let mut i: usize = 0;
    let mut lexs: Vec<LexType> = Vec::new();
    while tokens.get(i).is_some() {
        let t = &tokens[i];
        if t.token_type == TokenType::TokenFunStart {
            i += 1;
            if tokens[i].token_type == TokenType::TokenFunName {
                let mut args: Vec<ArgumentDef> = Vec::new();
                let mut scope: Vec<LexType> = Vec::new();
                let name = tokens[i].value.clone();
                i += 1;
                while tokens[i].token_type == TokenType::TokenFunArgKey && tokens[i+1].token_type == TokenType::TokenFunArgValue {
                    let argtype: Type;
                    if tokens[i+1].value == "int" {
                    argtype = Type::Int;
                    } else if tokens[i+1].value == "string" {
                    argtype = Type::String;
                    } else if tokens[i+1].value == "bool" {
                    argtype = Type::Bool;
                    } else if tokens[i+1].value == "float" {
                    argtype = Type::Float;
                    } else {
                        panic!("error when parse type")
                    }
                    args.push(ArgumentDef{ name: tokens[i].value.to_string(), argtype});
                    i += 2;
                }
                let i2 = i;
                while tokens[i].token_type != TokenType::TokenScopeEnd {
                    i += 1;
                }
                scope.extend(parse(&tokens[i2..i]));
                lexs.push(LexType::Function(name, args, scope));
            }
        } else if t.token_type == TokenType::TokenVarName {
            let name = t.value.clone();
            i += 4;
            let typearg: Type;
            let rvalue = &tokens[i+1].value;
            let value: Value;
            if tokens[i-2].value == "string" {
                typearg = Type::String;
                value = Value::String(rvalue.to_owned());
            } else if tokens[i-2].value == "int" {
                typearg = Type::Int;
                value = Value::Int(rvalue.parse().expect("error while parsing int"))
            } else if tokens[i-2].value == "bool" {
                typearg = Type::Bool;
                value = Value::Bool(rvalue == "true")
            } else if tokens[i-2].value == "float" {
                typearg = Type::Float;
                value = Value::Float(rvalue.parse().expect("error while parsing float"))
            } else {
                panic!("error when parse type")
            }
            lexs.push(LexType::Variable(name, typearg, value));
        } else if t.token_type == TokenType::TokenFunCall {
            let mut arguments: Vec<Value> = Vec::new();
            i += 1;
            while tokens[i].token_type == TokenType::TokenFunCallArg {
                match &tokens[i].value.parse::<isize>() {
                    Ok(a) => {
                        arguments.push(Value::Int(a.to_owned()));
                        i += 1;
                        continue;
                    }, Err(_) => {}
                }
                match &tokens[i].value.parse::<f64>() {
                    Ok(a) => {
                        arguments.push(Value::Float(a.to_owned()));
                        i += 1;
                        continue;
                    }, Err(_) => {}
                }
                if &tokens[i].value == "true" {
                    arguments.push(Value::Bool(true))
                } else if &tokens[i].value == "false" {
                    arguments.push(Value::Bool(false))
                } else {
                    arguments.push(Value::String(tokens[i].value.clone()))
                }
                i += 1;
            }
            lexs.push(LexType::FunctionCall(t.value.clone()[1..t.value.len()].to_string(), arguments))
        }
        i += 1;
    }
    lexs
}

fn run(
    lexs: &Vec<LexType>,
    functions: Option<HashMap<String, Function>>,
    variables: Option<HashMap<String, &Value>>
) {
    let mut functions: HashMap<String, Function> = functions.unwrap_or_default();
    let mut variables: HashMap<String, &Value> = variables.unwrap_or_default();
    for lex in lexs {
        match lex {
            LexType::Function(a, b, c) => {
                functions.insert(a.to_string(), Function{args: b, scope: c});
            },
            LexType::Variable(a, _, c) => {
                variables.insert(a.to_string(), c);
            },
            LexType::FunctionCall(a, b) => {
                if a == "say" {
                    match b[0] {
                        Value::String(ref a) => {
                            let mut b = a.clone();
                            while b.find('@').is_some() && b.find('*').is_some() {
                                let r1 = b.find('@').unwrap();
                                let r2 = b.find('*').unwrap();
                                let value = match variables[&b[r1+1..r2].to_string()] {
                                    Value::String(a) => a.to_string(),
                                    Value::Bool(a) => a.to_string(),
                                    Value::Float(a) => a.to_string(),
                                    Value::Int(a) => a.to_string()
                                };
                                b.replace_range(r1..r2+1, &value);
                            }
                            println!("{b}");
                        },
                        _ => { panic!("param isn't string") }
                    }
                } else {
                    match functions.get(a) {
                        None => {
                            panic!("function not found {a}");
                        },
                        Some(a) => {
                            let mut arguments: HashMap<String, &Value> = HashMap::new();
                            for (i, i2) in zip(a.args, b) {
                                arguments.insert(i.name.to_string(), i2);
                            }
                            run(a.scope, Some(functions.clone()), Some(arguments));
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let tmp = args().collect::<Vec<String>>();
    let filename: String = tmp[1].to_owned();
    let filecontent: Vec<String> = read_to_string(filename).unwrap().chars().map(|c| c.to_string()).collect();
    let tokens = tokenizer(filecontent);
    for i in &tokens {
        println!("type: {:#?}, buffer: {:#?}", i.token_type, i.value);
    }
    let lexs = parse(&tokens);
    for i in &lexs {
        println!("lextype: {:#?}", i);
    }
    run(&lexs, None, None);
}
