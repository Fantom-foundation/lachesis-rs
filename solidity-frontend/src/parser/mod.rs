mod non_empty;
use crate::parser::non_empty::NonEmpty;

pub enum SourceUnit {
    PragmaDirective(Identifier),
    ImportDirective(ImportDirective),
    ContractDefinition(ContractDefinition),
}

pub struct ContractDefinition {
    contract_parts: Vec<ContractPart>,
    contract_type: ContractType,
    inheritance_specifiers: Vec<InheritanceSpecifier>,
    name: Identifier,
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
    name: Identifier,
    values: Vec<Identifier>,
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
    Addreses,
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
