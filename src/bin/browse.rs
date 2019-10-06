use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{SelectView, TextView};
use cursive::theme::{BaseColor, ColorStyle};
use cursive::utils::markup::StyledString;
use async_std::task::block_on;
use futures::stream::TryStreamExt;
use goophy::{Locator, Entry};

fn main() {
    block_on(async_main()).unwrap();
}

async fn async_main() -> goophy::Result {
    let locator = Locator::root();
    let dir = goophy::get_directory("floodgap.com", 70, &locator)
        .inspect_ok(|entry| println!("< {}", entry.label))
        .try_collect::<Vec<_>>().await?;

    let mut siv = Cursive::termion().unwrap();

    siv.add_fullscreen_layer(menu_layer(dir).full_screen());
    siv.add_global_callback('q', |s| s.quit());
    siv.run();

    Ok(())
}

fn menu_layer(entries: impl IntoIterator<Item=Entry>) -> impl View {
    let items = entries
        .into_iter()
        .map(|entry| match entry.kind {
            '0' => {
                let color_style = ColorStyle::from(BaseColor::Blue);
                let label = StyledString::styled(&entry.label, color_style);
                (label, entry)
            }
            '1' => {
                let color_style = ColorStyle::from(BaseColor::Blue);
                let label = StyledString::styled(&entry.label, color_style);
                (label, entry)
            }
            _ => (StyledString::plain(&entry.label), entry)
        });
    
    SelectView::<Entry>::new()
        .with_all(items)
        .on_submit(handle_entry_submit)
        .scrollable()
        .scroll_x(true)
}

fn handle_entry_submit(siv: &mut Cursive, entry: &Entry) {
    match entry.kind {
        '0' => block_on(async {
            let text = goophy::get_text_file(&entry.host, entry.port, &entry.locator)
                .await.unwrap()
                .join("\n");
        
            siv.pop_layer();
            siv.add_fullscreen_layer(
                TextView::new(text)
                .scrollable()
                .scroll_x(true)
            );
        }),
        '1' => block_on(async {
            let dir = goophy::get_directory(&entry.host, entry.port, &entry.locator)
                .try_collect::<Vec<_>>().await.unwrap();

            siv.pop_layer();
            siv.add_fullscreen_layer(menu_layer(dir).full_screen());
        }),
        _ => {},
    }
}
