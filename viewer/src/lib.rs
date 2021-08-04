pub mod app;
mod image_manager;
mod presenter;
mod shader;
mod vertex;
mod viewer;

pub fn hello_from_viewer() {
    println!("Hello world from viewer");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
