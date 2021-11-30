
pub struct Diagnostic {
    pub code: u16,
    pub message: &'static str
}

macro_rules! make_diagnostics {
    ($([$name:ident, $code:expr, $msg:expr]),+) => {
        $(
            pub const $name: Diagnostic = Diagnostic {
                code: $code,
                message: $msg
            };
        )+
    }
}

pub struct Diagnostics;

impl Diagnostics {

    make_diagnostics!([
        END_OF_STR,
        1001,
        "Expected end of string."
    ], [
        DECIMAL_POINT,
        1002,
        "Floating points cannot contain more than one decimal point."
    ], [
        EXPECTED_PROP_NAME,
        1003,
        "Expected a property name."
    ], [
        INVALID_CHAR,
        1004,
        "Invalid character $."
    ], [
        UNEXPECTED_OP,
        1005,
        "Unexpected operator $."
    ], [
        UNEXPECTED_PUNC,
        1006,
        "Unexpected punctuation $."
    ], [
        SEMICOLON,
        1007,
        "Expected semicolon at the end of the expression."
    ], [
        END_OF_BLOCK,
        1008,
        "Expected end of block."
    ], [
        EXPECTED_FOUND,
        1009,
        "Expected $, but found $."
    ], [
        START_OF_BLOCK,
        1010,
        "Expected start of block."
    ], [
        ARROW_ACCESS,
        1011,
        "Arrow access cannot be chained."
    ], [
        EXPECTED_DELIMITER,
        1012,
        "Expected delimiter $."
    ], [
        TOO_MUCH_TYPES,
        1013,
        "Too much typings provided, expected only $."
    ], [
        EMPTY_CHAR_LITERAL,
        1014,
        "Empty character literal."
    ], [
        CONST_WITHOUT_INIT,
        1015,
        "Constant variables must have an initializer."
    ], [
        NO_GENERICS,
        1016,
        "Generics are not allowed here."
    ], [
        END_OF_ITER,
        1017,
        "Expected end of iterator."
    ], [
        DISALLOWED,
        1018,
        "$ is not allowed here."
    ], [
        MANY_ENTRIES,
        1019,
        "Too many entry points. There can be only one."
    ], [
        WRONG_MATCH_ARM_EXP,
        1020,
        "Incorrect match arm expression. Match arms only accept enum variants or literals."
    ], [
        ALREADY_HAS_MODIFIER,
        1021,
        "The field is already $. Unnecessary modifier."
    ], [
        CONFUSABLE,
        1022,
        "Found $, which is very similar to $."
    ], [
        INVALID_DIGIT,
        1023,
        "Invalid digit."
    ], [
        POINTLESS_TEMPLATE,
        1024,
        "Pointless template literls. Use normal string literals instead."
    ], [
        ONE_CHAR_ENDPOINT,
        1025,
        "Expected character to contain only one codepoint."
    ], [
        EXPECTED,
        1026,
        "Expected $."
    ], [
        UNEXPECTED,
        1027,
        "Unexpected $."
    ], [
        UNEXPECTED_EOF,
        1028,
        "Unexpected end of file."
    ], [
        TYPE_NOT_FOUND_FROM_MOD,
        2001,
        "Type $ not found from module $"
    ]
);

}

pub fn format_diagnostic(diagnostic: &Diagnostic, vars: Vec<&str>) -> String {
    let msg = diagnostic.message;
    if vars.is_empty() { return msg.to_string() };
    let mut ind: usize = 0;
    let mut new_str = String::new();
    for ch in msg.chars() {
        if ch == '$' {
            new_str.push_str(&vars[ind]);
            ind += 1;
        } else {
            new_str.push(ch)
        }
    }
    new_str
}