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

pub enum Expression {
    BinaryExpression(BinaryExpression),
    GroupExpression(Box<Expression>),
    IndexAccess(Box<Expression>, Box<Expression>),
    LeftUnaryExpression(LeftUnaryExpression),
    MemberAccess(Box<Expression>, Identifier),
    NewExpression(TypeName),
    PrimaryExpression(PrimaryExpression),
    RightUnaryExpression(RightUnaryExpression),
    TernaryOperator(Box<Expression>, Box<Expression>, Box<Expression>),
}

pub struct FunctionCall {
    callee: Identifier,
    arguments: Vec<Expression>,
}

pub struct Identifier(String);

pub enum TypeName {
    ElementaryTypeName(ElementaryTypeName),
}

pub struct BinaryExpression {
    left: Expression,
    op: BinaryOperator,
    right: Expression,
}

pub struct LeftUnaryExpression {
    value: Expression,
    op: LeftUnaryOperator,
}

pub struct RightUnaryExpression {
    value: Expression,
    op: RightUnaryOperator,
}

pub enum PrimaryExpression {
    BooleanLiteral(bool),
    ElementaryTypeName(ElementaryTypeName),
    HexLiteral(String),
    Identifier(Identifier),
    NumberLiteral {
        value: String,
        unit: Option<NumberUnit>,
    },
    StringLiteral(String),
    TupleExpression(Vec<Expression>),
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
    Addreses,
    Bool,
    Byte(u8),
    Fixed(u8, u8),
    Int(u8),
    String,
    Uint(u8),
    Ufixed(u8, u8),
}
