pub mod arg_parse;
pub mod config;
pub mod doc;
pub mod docs;
pub mod io;
pub mod json;
pub use tap::pipe;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
