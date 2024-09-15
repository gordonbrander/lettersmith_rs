pub mod cli;
pub mod config;
pub mod doc;
pub mod docs;
pub mod io;
pub mod json;
pub mod markdown;
pub mod stub;
pub use tap::pipe;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
