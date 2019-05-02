mod non_empty;
use crate::parser::non_empty::NonEmpty;
use std::str::FromStr;

#[inline]
fn is_ascii_hexdigit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

#[inline]
fn is_valid_string_character(c: char) -> bool {
    c != '\n' && c != '\r' && c != '"'
}

#[inline]
fn is_first_digit_identifier(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$'
}

#[inline]
fn is_digit_identifier(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

#[inline]
fn vec_to_string(s: Vec<char>) -> String {
    s.into_iter().collect::<String>()
}

named!(expression<&str, Expression>, alt_complete!(
    right_unary_expression | new_expression | index_access | member_access | function_call_expression |
    group_expression | left_unary_expression | power_expression | priority_two_binary_expression |
    priority_three_binary_expression | priority_four_binary_expression | and_binary_expression |
    xor_binary_expression | or_binary_expression | comparison_binary_expression |
    equality_binary_expression | logical_and_expression | logical_or_expression |
    ternary_operator_expression | assignment_binary_expression | primary_expression_expression
));

named!(primary_expression<&str, PrimaryExpression>, alt_complete!(
    elementary_type_name => {|t| PrimaryExpression::ElementaryTypeName(t)} |
    identifier => {|i| PrimaryExpression::Identifier(i)} |
    literal => {|l| PrimaryExpression::Literal(l)}
));

named!(literal<&str, Literal>, alt_complete!(boolean_literal | string_literal | number_literal | hex_literal));

named!(type_name<&str, TypeName>, alt_complete!(
    elementary_type_name => {|e| TypeName::ElementaryTypeName(e)}
));

named!(assignment_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
    op:    assignment_operators >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }))
)));
named!(assignment_operators<&str, BinaryOperator>, alt_complete!(
    ws!(tag!("|=")) => {|_| BinaryOperator::BarEquals} |
    ws!(tag!("^=")) => {|_| BinaryOperator::HatEquals} |
    ws!(tag!("&=")) => {|_| BinaryOperator::AmpersandEquals} |
    ws!(tag!("<<=")) => {|_| BinaryOperator::DoubleLesserThanEquals} |
    ws!(tag!(">>=")) => {|_| BinaryOperator::DoubleBiggerThanEquals} |
    ws!(tag!("+=")) => {|_| BinaryOperator::PlusEquals} |
    ws!(tag!("-=")) => {|_| BinaryOperator::DashEquals} |
    ws!(tag!("*=")) => {|_| BinaryOperator::StarEquals} |
    ws!(tag!("/=")) => {|_| BinaryOperator::SlashEquals} |
    ws!(tag!("%=")) => {|_| BinaryOperator::PercentEquals} |
    ws!(tag!("=")) => {|_| BinaryOperator::Equals}
));
named!(ternary_operator_expression<&str, Expression>, ws!(do_parse!(
    condition:   expression >>
    if_branch:   expression >>
    else_branch: expression >>
    (Expression::TernaryOperator(Box::new(condition), Box::new(if_branch), Box::new(else_branch)))
)));
named!(logical_or_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
           ws!(tag!("||")) >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op: BinaryOperator::DoubleBar,
        right: Box::new(right),
    }))
)));
named!(logical_and_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
           ws!(tag!("&&")) >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op: BinaryOperator::DoubleAmpersand,
        right: Box::new(right),
    }))
)));
named!(equality_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
    op:    equality_operators >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }))
)));
named!(equality_operators<&str, BinaryOperator>, alt_complete!(
    ws!(tag!("==")) => {|_| BinaryOperator::DoubleEquals} |
    ws!(tag!("!=")) => {|_| BinaryOperator::BangEquals}
));
named!(comparison_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
    op:    comparison_binary_operators >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }))
)));
named!(comparison_binary_operators<&str, BinaryOperator>, alt_complete!(
    ws!(tag!(">")) => {|_| BinaryOperator::BiggerThan} |
    ws!(tag!(">=")) => {|_| BinaryOperator::BiggerOrEqualsThan} |
    ws!(tag!("<")) => {|_| BinaryOperator::LesserThan} |
    ws!(tag!("<=")) => {|_| BinaryOperator::LesserOrEqualsThan}
));
named!(or_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
           tag!("|") >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op: BinaryOperator::Bar,
        right: Box::new(right),
    }))
)));
named!(xor_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
           tag!("^") >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op: BinaryOperator::Hat,
        right: Box::new(right),
    }))
)));
named!(and_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
           tag!("&") >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op: BinaryOperator::Ampersand,
        right: Box::new(right),
    }))
)));
named!(priority_four_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
    op:    priority_four_operators >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }))
)));
named!(priority_four_operators<&str, BinaryOperator>, alt_complete!(
    ws!(tag!("<<")) => {|_| BinaryOperator::DoubleLesserThan} |
    ws!(tag!(">>")) => {|_| BinaryOperator::DoubleBiggerThan}
));

named!(priority_three_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
    op:    priority_three_operators >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }))
)));
named!(priority_three_operators<&str, BinaryOperator>, alt_complete!(
    tag!("+") => {|_| BinaryOperator::Plus} |
    tag!("-") => {|_| BinaryOperator::Dash}
));
named!(primary_expression_expression<&str, Expression>, do_parse!(
    expression: primary_expression >>
    (Expression::PrimaryExpression(expression))
));
named!(priority_two_binary_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
    op:    priority_two_operators >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }))
)));
named!(priority_two_operators<&str, BinaryOperator>, alt_complete!(
    tag!("%") => {|_| BinaryOperator::Percent} |
    tag!("*") => {|_| BinaryOperator::Star} |
    tag!("/") => {|_| BinaryOperator::Slash}
));
named!(power_expression<&str, Expression>, ws!(do_parse!(
    left:  expression >>
           tag!("**") >>
    right: expression >>
    (Expression::BinaryExpression(BinaryExpression {
        left: Box::new(left),
        op: BinaryOperator::DoubleStar,
        right: Box::new(right),
    }))
)));
named!(right_unary_operator<&str, RightUnaryOperator>, alt_complete!(
    ws!(tag!("--")) => {|o| RightUnaryOperator::DoubleDash} |
    ws!(tag!("++")) => {|o| RightUnaryOperator::DoublePlus}
));
named!(right_unary_expression<&str, Expression>, ws!(do_parse!(
    value: expression >>
    op:    right_unary_operator >>
    (Expression::RightUnaryExpression(RightUnaryExpression {value: Box::new(value), op}))
)));
named!(group_expression<&str, Expression>, ws!(do_parse!(
                tag!("(") >>
    expression: expression >>
                tag!(")") >>
    (Expression::GroupExpression(Box::new(expression)))
)));
named!(function_call_expression<&str, Expression>, ws!(do_parse!(
    callee: expression >>
    args:   function_call_arguments >>
    (Expression::FunctionCall(FunctionCall {callee: Box::new(callee), arguments: args}))
)));
named!(function_call_arguments<&str, FunctionCallArguments>, alt_complete!(
    name_value_list => {|e| FunctionCallArguments::NameValueList(e)} |
    expression_list => {|e| FunctionCallArguments::ExpressionList(e)}
));
named!(expression_list<&str, Vec<Box<Expression>>>, ws!(do_parse!(
    first: expression >>
    tail:  expression_tail >>
    ({
        let mut result = vec![Box::new(first)];
        result.extend(tail);
        result
    })
)));
named!(expression_tail_element<&str, Expression>, ws!(do_parse!(
                tag!(",") >>
    expression: expression >>
    (expression)
)));
named!(expression_tail<&str, Vec<Box<Expression>>>, ws!(fold_many0!(
    expression_tail_element, Vec::new(), |mut acc: Vec<_>, item: Expression| {
        acc.push(Box::new(item));
        acc
    }
)));
named!(name_value_list<&str, Vec<NameValue>>, ws!(do_parse!(
    first: name_value >>
    tail:  name_value_tail >>
    ({
        let mut result = vec![first];
        result.extend(tail);
        result
    })
)));
named!(name_value_tail_element<&str, NameValue>, ws!(do_parse!(
                tag!(",") >>
    name_value: name_value >>
    (name_value)
)));
named!(name_value_tail<&str, Vec<NameValue>>, ws!(fold_many0!(
    name_value_tail_element, Vec::new(), |mut acc: Vec<_>, item: NameValue| {
        acc.push(item);
        acc
    }
)));
named!(name_value<&str, NameValue>, ws!(do_parse!(
    parameter: identifier >>
               tag!(":") >>
    value:     expression >>
    (NameValue {parameter, value: Box::new(value)})
)));
named!(member_access<&str, Expression>, ws!(do_parse!(
    parent: expression >>
            tag!(".") >>
    member: identifier >>
    (Expression::MemberAccess(Box::new(parent), member))
)));
named!(index_access<&str, Expression>, ws!(do_parse!(
    parent: expression >>
            tag!("[") >>
    member: expression >>
            tag!("]") >>
    (Expression::IndexAccess(Box::new(parent), Box::new(member)))
)));
named!(new_expression<&str, Expression>, ws!(do_parse!(
    type_name: type_name >>
    (Expression::NewExpression(type_name))
)));
named!(left_unary_operator<&str, LeftUnaryOperator>, alt_complete!(
    tag!("!") => {|o| LeftUnaryOperator::Bang} |
    tag!("-") => {|o| LeftUnaryOperator::Dash} |
    ws!(tag!("delete")) => {|o| LeftUnaryOperator::Delete} |
    ws!(tag!("--")) => {|o| LeftUnaryOperator::DoubleDash} |
    ws!(tag!("++")) => {|o| LeftUnaryOperator::DoublePlus} |
    ws!(tag!("~")) => {|o| LeftUnaryOperator::Home} |
    ws!(tag!("+")) => {|o| LeftUnaryOperator::Plus}
));
named!(left_unary_expression<&str, Expression>, ws!(do_parse!(
    value: expression >>
    op:    left_unary_operator >>
    (Expression::LeftUnaryExpression(LeftUnaryExpression {value: Box::new(value), op}))
)));

named!(number_unit<&str, NumberUnit>, alt_complete!(
    ws!(tag!("wei")) => {|_| NumberUnit::Wei} |
    ws!(tag!("szabo")) => {|_| NumberUnit::Szabo} |
    ws!(tag!("finney")) => {|_| NumberUnit::Finney} |
    ws!(tag!("ether")) => {|_| NumberUnit::Ether} |
    ws!(tag!("seconds")) => {|_| NumberUnit::Seconds} |
    ws!(tag!("minutes")) => {|_| NumberUnit::Minutes} |
    ws!(tag!("hours")) => {|_| NumberUnit::Hours} |
    ws!(tag!("days")) => {|_| NumberUnit::Days} |
    ws!(tag!("weeks")) => {|_| NumberUnit::Weeks} |
    ws!(tag!("years")) => {|_| NumberUnit::Years}
));
named!(full_dec_number<&str, String>, do_parse!(
    number:   no_exp_dec_number >>
    e_part:   ws!(one_of!("eE")) >>
    exp_part: many1!(one_of!("0123456789")) >>
    (format!("{}{}{}", number, e_part, vec_to_string(exp_part)))
));
named!(no_exp_dec_number<&str, String>, alt_complete!(
    float_dec_number | simple_dec_number
));
named!(float_dec_number<&str, String>, do_parse!(
    int_part: many1!(one_of!("0123456789")) >>
              tag!(".") >>
    dec_part: many0!(one_of!("0123456789")) >>
    (format!("{}.{}", vec_to_string(int_part), vec_to_string(dec_part)))
));
named!(simple_dec_number<&str, String>, do_parse!(
    int_part: many1!(one_of!("0123456789")) >>
    (vec_to_string(int_part))
));
named!(dec_number<&str, String>, alt_complete!(full_dec_number | no_exp_dec_number));
named!(hex_number<&str, String>, do_parse!(
             tag!("0x") >>
    content: take_while!(is_ascii_hexdigit) >>
    (content.to_owned())
));
named!(number_literal_number<&str, String>, alt_complete!(dec_number | hex_number));
named!(number_literal_no_unit<&str, Literal>, do_parse!(
    number: number_literal_number >>
    (Literal::NumberLiteral {value: number, unit: None})
));
named!(number_literal_unit<&str, Literal>, do_parse!(
    number: number_literal_number >>
            many0!(one_of!(" \t\n")) >>
    unit:   number_unit >>
    (Literal::NumberLiteral {value: number, unit: Some(unit)})
));
named!(number_literal<&str, Literal>, alt_complete!(number_literal_unit | number_literal_no_unit));
named!(string_quotes<&str, char>, one_of!("'\""));
named!(string_literal<&str, Literal>, do_parse!(
             string_quotes >>
    content: take_while!(is_valid_string_character) >>
             string_quotes >>
    (Literal::StringLiteral(content.to_owned()))
));
named!(hex_pair<&str, Vec<char>>, many_m_n!(2, 2, one_of!("0123456789abcdefABCDEF")));
named!(hex_literal<&str, Literal>, do_parse!(
             tag!("hex") >>
             string_quotes >>
    content: many0!(hex_pair) >>
             string_quotes >>
    (Literal::HexLiteral(vec_to_string(content.into_iter().flatten().collect::<Vec<char>>())))
));
named!(identifier<&str, Identifier>, do_parse!(
    first_char: one_of!("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$") >>
    rest:       take_while!(is_digit_identifier) >>
    ({
        let mut s = first_char.to_string();
        s.push_str(rest);
        Identifier(s)
    })
));
named!(boolean_literal<&str, Literal>, alt_complete!(
    tag!("true") => {|_| Literal::BooleanLiteral(true)} |
    tag!("false") => {|_| Literal::BooleanLiteral(false)}
));
named!(get_u8<&str, u8>, do_parse!(
    digits: many1!(one_of!("0123456789")) >>
    (u8::from_str(digits.into_iter().collect::<String>().as_str()).unwrap())
));
named!(byte_array_type_name<&str, ElementaryTypeName>, do_parse!(
            tag!("byte") >>
    length: get_u8 >>
    (ElementaryTypeName::Byte(length))
));
named!(fixed_type_name<&str, ElementaryTypeName>, do_parse!(
             tag!("fixed") >>
    length:  get_u8 >>
             tag!("x") >>
    decimal: get_u8 >>
    (ElementaryTypeName::Fixed(length, decimal))
));
named!(ufixed_type_name<&str, ElementaryTypeName>, do_parse!(
             tag!("ufixed") >>
    length:  get_u8 >>
             tag!("x") >>
    decimal: get_u8 >>
    (ElementaryTypeName::Ufixed(length, decimal))
));
named!(valid_integer_size<&str, u8>, alt_complete!(
    tag!("8") => {|_| 8} |
    tag!("16") => {|_| 16} |
    tag!("24") => {|_| 24} |
    tag!("32") => {|_| 32} |
    tag!("40") => {|_| 40} |
    tag!("48") => {|_| 48} |
    tag!("56") => {|_| 56} |
    tag!("64") => {|_| 64} |
    tag!("72") => {|_| 72} |
    tag!("80") => {|_| 80} |
    tag!("88") => {|_| 88} |
    tag!("96") => {|_| 96} |
    tag!("104")  => {|_| 104} |
    tag!("112")  => {|_| 112} |
    tag!("120")  => {|_| 120} |
    tag!("128")  => {|_| 128} |
    tag!("136")  => {|_| 136} |
    tag!("144")  => {|_| 144} |
    tag!("152")  => {|_| 152} |
    tag!("160")  => {|_| 160} |
    tag!("168")  => {|_| 168} |
    tag!("176")  => {|_| 176} |
    tag!("184")  => {|_| 184} |
    tag!("192")  => {|_| 192} |
    tag!("200")  => {|_| 200} |
    tag!("208")  => {|_| 208} |
    tag!("216")  => {|_| 216} |
    tag!("224")  => {|_| 224} |
    tag!("232")  => {|_| 232} |
    tag!("240")  => {|_| 240} |
    tag!("248")  => {|_| 248} |
    tag!("256")  => {|_| 0}
));
named!(sized_int_type_name<&str, ElementaryTypeName>, do_parse!(
          tag!("int") >>
    size: valid_integer_size >>
    (ElementaryTypeName::Int(size))
));
named!(sized_uint_type_name<&str, ElementaryTypeName>, do_parse!(
          tag!("uint") >>
    size: valid_integer_size >>
    (ElementaryTypeName::Uint(size))
));
named!(elementary_type_name<&str, ElementaryTypeName>, alt_complete!(
    tag!("address") => {|_| ElementaryTypeName::Address} |
    tag!("string") => {|_| ElementaryTypeName::String} |
    tag!("bool") => {|_| ElementaryTypeName::Bool} |
    tag!("byte") => {|_| ElementaryTypeName::Byte(1)} |
    byte_array_type_name => {|e| e} |
    tag!("ufixed") => {|_| ElementaryTypeName::Ufixed(128, 18)} |
    ufixed_type_name => {|e| e} |
    tag!("fixed") => {|_| ElementaryTypeName::Fixed(128, 18)} |
    fixed_type_name => {|e| e} |
    tag!("int") => {|_| ElementaryTypeName::Int(128)} |
    sized_int_type_name => {|e| e} |
    tag!("uint") => {|_| ElementaryTypeName::Uint(128)} |
    sized_uint_type_name => {|e| e}
));

pub struct Program(pub Vec<SourceUnit>);

pub enum SourceUnit {
    PragmaDirective(Identifier),
    ImportDirective(ImportDirective),
    ContractDefinition(ContractDefinition),
}

pub struct ContractDefinition {
    pub contract_parts: Vec<ContractPart>,
    contract_type: ContractType,
    inheritance_specifiers: Vec<InheritanceSpecifier>,
    pub name: Identifier,
}

pub enum TypeName {
    Address,
    AddressPayable,
    ArrayTypeName(Box<TypeName>, Option<Box<Expression>>),
    ElementaryTypeName(ElementaryTypeName),
    FunctionTypeName(FunctionTypeName),
    Mapping(ElementaryTypeName, Box<TypeName>),
    UserDefinedTypeName(UserDefinedTypeName),
}

pub type Block = Vec<Statement>;
pub enum Statement {
    Block(Block),
    Break,
    Continue,
    DoWhileStatement(Box<Statement>, Expression),
    Emit(FunctionCall),
    ForStatement(SimpleStatement, Expression, Expression, Box<Statement>),
    IfStatement(IfStatement),
    InlineAssemblyStatement(Option<String>, AssemblyBlock),
    PlaceholderStatement,
    Return(Option<Expression>),
    SimpleStatement(SimpleStatement),
    Throw,
    WhileStatement(Expression, Box<Statement>),
}

pub enum SimpleStatement {
    ExpressionStatement(Expression),
    VariableDefinition(Vec<VariableDeclaration>, Option<Expression>),
}

pub enum Expression {
    BinaryExpression(BinaryExpression),
    FunctionCall(FunctionCall),
    GroupExpression(Box<Expression>),
    IndexAccess(Box<Expression>, Box<Expression>),
    LeftUnaryExpression(LeftUnaryExpression),
    MemberAccess(Box<Expression>, Identifier),
    NewExpression(TypeName),
    PrimaryExpression(PrimaryExpression),
    RightUnaryExpression(RightUnaryExpression),
    TernaryOperator(Box<Expression>, Box<Expression>, Box<Expression>),
}

pub enum ContractType {
    Contract,
    Interface,
    Library,
}

pub struct InheritanceSpecifier {
    parent: UserDefinedTypeName,
    arguments: Vec<Expression>,
}

pub enum ContractPart {
    EnumDefinition(EnumDefinition),
    EventDefinition(EventDefinition),
    FunctionDefinition(FunctionDefinition),
    ModifierDefinition(ModifierDefinition),
    StateVariableDeclaration(StateVariableDeclaration),
    StructDefinition(StructDefinition),
    UsingForDeclaration(UsingForDeclaration),
}

pub struct StructDefinition {
    name: Identifier,
    variables: NonEmpty<VariableDeclaration>,
}

pub struct ModifierDefinition {
    name: Identifier,
    parameters: Option<Vec<Parameter>>,
}

pub struct FunctionDefinition {
    body: Option<Block>,
    modifiers: FunctionDefinitionModifier,
    name: Option<Identifier>,
    parameters: Vec<Parameter>,
    return_values: Vec<Parameter>,
}

pub struct EventDefinition {
    anonymous: bool,
    name: Identifier,
    parameters: Vec<EventParameter>,
}

pub struct EnumDefinition {
    pub name: Identifier,
    pub values: Vec<Identifier>,
}

pub struct EventParameter {
    indexed: bool,
    name: Option<Identifier>,
    type_name: TypeName,
}

pub enum FunctionDefinitionModifier {
    External,
    Internal,
    ModifierInvocation(ModifierInvocation),
    Private,
    Public,
    StateMutability(StateMutability),
}

pub struct ModifierInvocation {
    name: Identifier,
    arguments: Vec<Expression>,
}

pub struct Parameter {
    identifier: Option<Identifier>,
    storage: Option<Storage>,
    type_name: TypeName,
}

pub enum UsingForDeclaration {
    UsingForAll(Identifier),
    UsingFor(Identifier, TypeName),
}

pub enum VariableModifier {
    Constant,
    Internal,
    Private,
    Public,
}

pub struct StateVariableDeclaration {
    pub type_name: TypeName,
    modifiers: Vec<VariableModifier>,
    name: Identifier,
    pub value: Option<Expression>,
}

pub struct UserDefinedTypeName {
    pub base: Identifier,
    members: Vec<Identifier>,
}

pub enum ImportDirective {
    SimpleImport(String, Option<Identifier>),
    ImportFrom(Vec<(Identifier, Option<Identifier>)>, String),
    ImportAllFrom(String, Box<Identifier>),
}

pub struct IfStatement {
    condition: Expression,
    true_branch: Box<Statement>,
    false_branch: Box<Statement>,
}

pub struct VariableDeclaration {
    identifier: Identifier,
    storage: Option<Storage>,
    type_name: TypeName,
}

pub enum LeftUnaryOperator {
    Bang,
    Delete,
    Dash,
    DoubleDash,
    DoublePlus,
    Home,
    Plus,
}

pub enum RightUnaryOperator {
    DoubleDash,
    DoublePlus,
}

pub enum BinaryOperator {
    Ampersand,
    AmpersandEquals,
    Bang,
    BangEquals,
    Bar,
    BarEquals,
    BiggerThan,
    BiggerOrEqualsThan,
    Dash,
    DashEquals,
    DoubleAmpersand,
    DoubleBar,
    DoubleStar,
    DoubleBiggerThan,
    DoubleBiggerThanEquals,
    DoubleEquals,
    DoubleLesserThan,
    DoubleLesserThanEquals,
    Equals,
    Hat,
    HatEquals,
    LesserThan,
    LesserOrEqualsThan,
    Percent,
    PercentEquals,
    Plus,
    PlusEquals,
    Slash,
    SlashEquals,
    Star,
    StarEquals,
}

pub struct FunctionCall {
    callee: Box<Expression>,
    arguments: FunctionCallArguments,
}

pub enum FunctionCallArguments {
    ExpressionList(Vec<Box<Expression>>),
    NameValueList(Vec<NameValue>),
}

pub struct NameValue {
    parameter: Identifier,
    value: Box<Expression>,
}

pub struct Identifier(String);

impl Identifier {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub enum StateMutability {
    Payable,
    Pure,
    View,
}

pub enum FunctionModifier {
    External,
    Internal,
    StateMutability(StateMutability),
}

pub enum Storage {
    Calldata,
    Memory,
    Storage,
}

pub struct FunctionTypeName {
    pub arguments: Vec<FunctionParameter>,
    modifiers: Vec<FunctionModifier>,
    pub return_values: Vec<FunctionParameter>,
}

pub struct FunctionParameter {
    pub type_name: TypeName,
    storage: Option<Storage>,
}

pub struct BinaryExpression {
    left: Box<Expression>,
    op: BinaryOperator,
    right: Box<Expression>,
}

pub struct LeftUnaryExpression {
    value: Box<Expression>,
    op: LeftUnaryOperator,
}

pub struct RightUnaryExpression {
    value: Box<Expression>,
    op: RightUnaryOperator,
}

pub enum PrimaryExpression {
    ElementaryTypeName(ElementaryTypeName),
    Identifier(Identifier),
    Literal(Literal),
    TupleExpression(Vec<Expression>),
}

pub enum Literal {
    BooleanLiteral(bool),
    HexLiteral(String),
    NumberLiteral {
        value: String,
        unit: Option<NumberUnit>,
    },
    StringLiteral(String),
}

pub enum NumberUnit {
    Wei,
    Szabo,
    Finney,
    Ether,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Years,
}

pub enum ElementaryTypeName {
    Address,
    Bool,
    Byte(u8),
    Fixed(u8, u8),
    Int(u8),
    String,
    Uint(u8),
    Ufixed(u8, u8),
}

type AssemblyBlock = Vec<Box<AssemblyStatement>>;
pub enum AssemblyStatement {
    AssemblyBlock(AssemblyBlock),
    AssemblyFunctionDefinition(AssemblyFunctionDefinition),
    AssemblyVariableDeclaration(AssemblyVariableDeclaration),
    AssemblyAssignment(AssemblyAssignment),
    AssemblyIf(AssemblyIf),
    AssemblyExpression(AssemblyExpression),
    AssemblySwitch(AssemblySwitch),
    AssemblyForLoop(AssemblyForLoop),
    Break,
    Continue,
}

pub struct AssemblyFunctionDefinition {
    block: AssemblyBlock,
    name: Identifier,
    parameters: Vec<Identifier>,
    return_values: Vec<Identifier>,
}

pub struct AssemblyVariableDeclaration {
    values: Vec<AssemblyExpression>,
    variables: NonEmpty<Identifier>,
}

pub struct AssemblyAssignment {
    expression: AssemblyExpression,
    variables: NonEmpty<Identifier>,
}

pub enum AssemblyExpression {
    AssemblyFunctionCall(AssemblyFunctionCall),
    Identifier(Identifier),
    Literal(Literal),
}

pub struct AssemblyIf {
    condition: AssemblyExpression,
    block: AssemblyBlock,
}

pub struct AssemblySwitch {
    condition: AssemblyExpression,
    body: AssemblySwitchBody,
}

pub enum AssemblySwitchBody {
    OnlyDefault(AssemblySwitchDefault),
    CaseList(NonEmpty<AssemblySwitchCase>, Option<AssemblySwitchDefault>),
}

pub struct AssemblySwitchDefault(AssemblyBlock);
pub struct AssemblySwitchCase(Literal, AssemblyBlock);

pub struct AssemblyForLoop {
    body: AssemblyBlock,
    increment_expressions: AssemblyBlock,
    init_values: AssemblyBlock,
    stop_condition: AssemblyExpression,
}

pub struct AssemblyFunctionCall {
    name: Identifier,
    arguments: Vec<AssemblyExpression>,
}
