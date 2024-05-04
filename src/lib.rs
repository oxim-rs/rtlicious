#![deny(missing_docs)]
//! Yosys RTLIL text representation parsing library.
//! ```
//! use rtlicious;
//! let src =
//! r#"module \test
//! wire $a;
//! end
//! "#;
//! let design = rtlicious::parse(src).unwrap();
//! assert_eq!(design.modules().len(), 1);
//! ```
mod attribute;
mod cell;
mod characters;
mod connect;
mod constant;
mod design;
mod identifier;
mod memory;
mod module;
mod process;
mod sigspec;
mod string;
mod switch;
mod sync;
mod value;
mod wire;

use std::collections::HashMap;

use getset::Getters;
use nom_locate::LocatedSpan;
use nom_tracable::TracableInfo;
use serde::Serialize;

/// A design is optional autoindex statement followed by zero or more modules.
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Design {
    /// The global autoindex value
    autoidx: Option<i32>,
    /// The modules in the design
    modules: HashMap<String, Module>,
}

/// Represents a module
/// A module is a collection of wires, memories, cells, processes, and connections
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Module {
    /// The attributes of the module
    attributes: HashMap<String, Constant>,
    /// The parameters of the module
    parameters: HashMap<String, Option<Constant>>,
    /// The wires of the module
    wires: HashMap<String, Wire>,
    /// The memories of the module
    memories: HashMap<String, Memory>,
    /// The cells of the module
    cells: HashMap<String, Cell>,
    /// The processes of the module
    processes: HashMap<String, Process>,
    /// The connections of the module
    connections: Vec<(SigSpec, SigSpec)>,
}

/// Represents a logic cell
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Cell {
    /// The type of the cell, ie. add, sub, etc.
    cell_type: String,
    /// The parameters of the cell
    parameters: HashMap<String, Constant>,
    /// The connections of the cell
    connections: HashMap<String, SigSpec>,
}

/// Represents a wire
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Wire {
    /// defaults to 1
    width: usize,
    /// defaults to 0
    offset: usize,
    /// if the wire is an input to the module
    input: bool,
    /// if the wire is an output to the module
    output: bool,
    /// if the wire is tristate?
    inout: bool,
    /// TODO: what is this?
    upto: bool,
    /// if the wire is signed? TODO: what is this?
    signed: bool,
    /// attributes of the wire
    attributes: HashMap<String, Constant>,
}

/// Represents a memory cell
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Memory {
    /// The width of the memory cell
    width: usize,
    /// The size of the memory cell
    size: usize,
    /// The offset of the memory cell
    offset: usize,
    /// The attributes of the memory cell
    attributes: HashMap<String, Constant>,
}

/// Represents a process
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Process {
    /// The attributes of the process
    attributes: HashMap<String, Constant>,
    /// The assignments of the process
    assignments: Vec<(SigSpec, SigSpec)>,
    /// The switch of the process
    switches: Vec<Switch>,
    /// The syncs of the process
    syncs: Vec<Sync>,
}

/// Constant enum
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Constant {
    /// Value variant, contains a vector of characters, ie. vec!['x', 'z', '1', 'm']
    Value(Vec<char>),
    /// Integer variant, contains an i32
    Integer(i32),
    /// String variant, contains a String
    String(String),
}

/// Represents a signal specification
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SigSpec {
    /// A constant value
    Constant(Constant),
    /// A wire id
    WireId(String),
    /// A range of bits from a wire
    Range(Box<SigSpec>, usize, Option<usize>),
    /// A concatenation of signals
    Concat(Vec<SigSpec>),
}

/// Represents a case body
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CaseBody {
    /// another switch, nested
    Switch(Switch),
    /// assign statement
    Assign((SigSpec, SigSpec)),
}

/// Represents a case
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Case {
    /// The attributes of the case
    pub(crate) attributes: HashMap<String, Constant>,
    /// The signals to compare against
    pub(crate) compare_against: Option<Vec<SigSpec>>,
    /// The body of the case
    pub(crate) case_bodies: Vec<CaseBody>,
}

/// Represents a switch
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Switch {
    /// The attributes of the switch
    pub(crate) attributes: HashMap<String, Constant>,
    /// The signal to switch on, ie. compare against
    pub(crate) switch_on_sigspec: SigSpec,
    /// run CaseBody if true
    pub(crate) cases: Vec<Case>,
}

/// Represents a sync statement
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SyncOn {
    /// Global sync
    Global,
    /// Initialization sync
    Init,
    /// Always sync
    Always,
    /// Signal sync
    Signal(SignalSync, SigSpec),
}

/// Represents a
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SignalSync {
    /// Low level sync
    Low,
    /// High level sync
    High,
    /// Posedge sync
    Posedge,
    /// Negedge sync
    Negedge,
    /// Edge sync
    Edge,
}

/// Represents a sync
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Sync {
    /// The sync event
    sync_event: SyncOn,
    /// The updates to apply on the sync event
    updates: Vec<(SigSpec, SigSpec)>,
    /// memwr statements
    memwrs: HashMap<String, Memwr>,
}

/// Represents a memwr statement
#[derive(Debug, Clone, PartialEq, Getters, Serialize)]
#[getset(get = "pub")]
pub struct Memwr {
    /// The attributes of the memwr
    attributes: HashMap<String, Constant>,
    /// The address of the memwr
    address: SigSpec,
    /// The data of the memwr
    data: SigSpec,
    /// The enable of the memwr
    enable: SigSpec,
    /// The priority mask of the memwr
    priority_mask: SigSpec,
}

/// Input type must implement trait Tracable
/// nom_locate::LocatedSpan<T, TracableInfo> implements it.
type Span<'a> = LocatedSpan<&'a str, TracableInfo>;

/// Parse a RTLIL design from a type that implements `AsRef<str>`.
pub fn parse(input: &str) -> Result<Design, Span> {
    Design::new_from_str(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sanity() {
        assert_eq!(1 + 1, 2);
    }
}
