use htom_core::home_page;
use htom_core::html_to_maud;
use polyester::page::Page;
use std::env;
use std::io;
use std::io::Read;

fn main() {
    let args_: Vec<String> = env::args().collect();
    let args: Vec<&str> = args_.iter().map(|s| s.as_ref()).collect();

    match args[1..] {
        ["home_page"] => {
            let page = home_page::HomePage { window_size: None };
            render_html(page);
        }

        ["convert"] => {
            let stdin = io::stdin();
            let mut html = String::new();
            stdin
                .lock()
                .read_to_string(&mut html)
                .expect("stdin read to end");

            let markup = html_to_maud::html_to_maud(
                &html,
                &html_to_maud::Config {
                    render: html_to_maud::Render::Auto,
                    id_style: html_to_maud::IdStyle::Full,
                    class_style: html_to_maud::ClassStyle::Full,
                },
            );
            println!("{}", markup);
        }

        _ => {
            println!("Invalid command, try 'page' or 'model' or 'convert'");
        }
    }
}

fn render_html<Model, Msg, AppEffect, Markup>(page: impl Page<Model, Msg, AppEffect, Markup>) {
    let (model, _effects) = page.init();
    let markup = page.view(&model);
    println!("{}", page.render_page(markup));
}
