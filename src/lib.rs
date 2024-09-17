pub use tap::pipe;
pub mod absolutize;
pub mod cli;
pub mod config;
pub mod doc;
pub mod docs;
pub mod io;
pub mod json;
pub mod liquid;
pub mod markdown;
pub mod permalink;
pub mod stub;
pub mod text;
pub mod token_template;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
