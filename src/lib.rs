use std::{collections, error, result};

pub mod inputs;

// it doesn't make sense to ever have both kinds of errors
// as it should be impossible for us to validate if we couldn't
// first succesfully parse the value
// this purposely doesn't impl std::error::Error to allow the from
// impls below.
//
// This makes sense as the primary use of this error is either:
// - the proc macro that goes from ui backing to regular struct
//      * this will probably output a regular Error impl
// - a display funciton
// both of which would consume the error rather than bubbling
// it up with a `?`
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Parse(Box<dyn error::Error + Sync + Send + 'static>),
    Validation(ValidationErrors),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error {}", e),
            Error::Validation(e) => write!(f, "validation error(s) {}", e),
        }
    }
}

type ValidationFn<T> = fn(&T) -> ValidationResult;

pub struct Validations<T> {
    funcs: Vec<ValidationFn<T>>,
}

impl<T> Validations<T> {
    pub fn new() -> Validations<T> {
        Validations { funcs: Vec::new() }
    }
    pub fn from_vec(funcs: Vec<ValidationFn<T>>) -> Self {
        Validations { funcs }
    }
    fn validate(&self, input: &T) -> result::Result<(), ValidationErrors> {
        let errors = self
            .funcs
            .iter()
            .map(|f| f(input))
            .filter_map(|r| if let Err(e) = r { Some(e) } else { None })
            .collect::<Vec<_>>();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors { errors })
        }
    }
}

#[derive(Debug)]
pub struct ValidationErrors {
    errors: Vec<String>,
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted =
            itertools::Itertools::intersperse(self.errors.iter().cloned(), ", ".to_string())
                .collect::<String>();
        write!(f, "[{}]", formatted)
    }
}

pub type ValidationResult = result::Result<(), String>;

pub type FormResult<T> = result::Result<T, FormError>;

#[derive(Debug)]
pub struct FormError {
    errors: collections::BTreeMap<&'static str, Error>,
}

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let iter = self
            .errors
            .iter()
            .map(|(field, issue)| format!("{}: {}", field, issue));
        write!(
            f,
            "{}",
            itertools::Itertools::intersperse(iter, ", ".to_string()).collect::<String>()
        )
    }
}

impl std::error::Error for FormError {}

impl FormError {
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
    pub fn new() -> FormError {
        FormError {
            errors: collections::BTreeMap::new(),
        }
    }
    pub fn add_result<T>(&mut self, field: &'static str, result: Result<T>) {
        if let Some(err) = result.err() {
            self.errors.insert(field, err);
        }
    }
    pub fn add_error(&mut self, field: &'static str, err: Error) {
        self.errors.insert(field, err);
    }
}

impl From<ValidationErrors> for Error {
    fn from(validation: ValidationErrors) -> Error {
        Error::Validation(validation)
    }
}
impl From<Vec<String>> for ValidationErrors {
    fn from(errors: Vec<String>) -> ValidationErrors {
        ValidationErrors { errors }
    }
}

impl<T> From<T> for Error
where
    T: error::Error + Sync + Send + 'static,
{
    fn from(t: T) -> Error {
        Error::Parse(Box::new(t))
    }
}

//  we force a custom result type here
//  as otherwise not much useful work is done within parse
//  and this also enables more useful use of the impls
pub type Result<T> = result::Result<T, Error>;

// principals of input backing:
// 1. View parts must be created manually by the user (to allow layout etc - implicit here is that it is up to the user to ensure
//    all inputs are created)
// 2. Prebuilt functions that create the inputs should be used to create the view
// 3. It should be difficult for the user to create inputs that message non-existant fields.
// 4. Have to allow for the fact that a single property may be inputted via multiple inputs (eg MonthYear created by a Month and
//    Year inputs)
// 5. Validatiors from the main struct should be automatically applied to the relevant fields of the input
// 6. Validators shoudl be able to be applied to each field whether or not all are valid.
// 7. Allow deriving ToInputMap and FromInputMap
// 8. Each datatype in the main struct should implement ui::Input trait to allow for ToInputMap to work
// 9. Message type should always be the same and then get mapped at a higher layer
// 10. ui::Input implementations should handle whether the input creates a single or multiple actual inputs and handle
//     the relevant messages

// user impls this on their local type, say 'DecimalInput'
// then the user impls the ui rendering as inherent fns on that type
// this is because that type may need to be rendered as multiple inputs
// user also defines a message type.
// The UserInput trait then ties together that Type with the Message type
// and provides a standard interface for updating, parsing and getting the input data
//
// While it would be nice to either require that `Output: Into<Self>` or have a `new(data:
// Self::Output) -> Self method here, this is too inflexible as the whole point of allowing the
// user to specify their own UserInput types is to allow them to have their own extra fields
// and behaviour that they want. This has the disadvantage of not allowing generation of the
// Self::Output -> Output function, but has the benefit of allowing the user to specify eg
// the options of a select at runtime (this is a fundamental requirement for `RelationalSelect`!)
//
/// By convention, users should create a `new` style function that takes a Self::Output, some other
/// stuff, and produces a Self.
pub trait UserInput {
    type Output;
    type Input;
    fn update(&mut self, input: Self::Input);
    fn parse(&self) -> Result<Self::Output>;
    fn set_value(&mut self, data: Self::Output);
}

// This can be optionally implemented, and provides a convenience
// for implementing
// this is most useful when the underlying type implments default
// as then there is no need to implement the new function
pub trait SetFromOutput<O>: UserInput<Output = O> {
    fn set_value(&mut self, data: O);
}

/// By convention, users should create a `new` style function that takes a Self::Output, some other
/// stuff, and produces a Self.
pub trait Form: Sized {
    type Msg;
    type Output;
    fn update(&mut self, input: Self::Msg);
    fn parse(&self) -> result::Result<Self::Output, FormError>;
}
