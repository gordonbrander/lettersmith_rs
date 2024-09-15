pub use tap::pipe;

pub mod cli;
pub mod config;
pub mod doc;
pub mod docs;
pub mod io;
pub mod json;
pub mod liquid;
pub mod markdown;
pub mod stub;
pub mod text;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
