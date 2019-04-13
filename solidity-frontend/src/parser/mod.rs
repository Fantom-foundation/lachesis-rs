pub enum SourceUnit {
    PragmaDirective(Identifier),
    ImportDirective(ImportDirective),
    ContractDefinition(ContractDefinition),
}

pub struct ContractDefinition {}

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
    InlineAssemblyStatement,
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

pub struct StructDefinition {}

pub struct ModifierDefinition {}

pub struct FunctionDefinition {}

pub struct EventDefinition {}

pub struct EnumDefinition {}

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
    type_name: TypeName,
    modifiers: Vec<VariableModifier>,
    name: Identifier,
    value: Option<Expression>,
}

pub struct UserDefinedTypeName {
    base: Identifier,
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
    callee: Identifier,
    arguments: Vec<Expression>,
}

pub struct Identifier(String);

pub enum FunctionModifier {
    External,
    Internal,
    Payable,
    Pure,
    View,
}

pub enum Storage {
    Calldata,
    Memory,
    Storage,
}

pub struct FunctionTypeName {
    arguments: Vec<FunctionParameter>,
    modifiers: Vec<FunctionModifier>,
    return_values: Vec<FunctionParameter>,
}

pub struct FunctionParameter {
    type_name: TypeName,
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
