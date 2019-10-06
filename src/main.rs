use async_std::task::block_on;
use futures::TryStreamExt;
use goophy::{Result, Locator};

fn main() -> Result {
    block_on(async_main())
}

async fn async_main() -> Result {
    let locator = "/gopher/proxy".parse().unwrap();
    print_text_file(&locator).await?;

    Ok(())
}

async fn print_text_file(locator: &Locator) -> Result {
    let file = goophy::get_text_file("floodgap.com", 70, &locator).await?;

    for line in file {
        println!("<| {}", line);
    }

    Ok(())
}

async fn print_directory(locator: &Locator) -> Result {
    let dir = goophy::get_directory("floodgap.com", 70, &locator);

    dir.try_for_each(|entry| async move {
        match entry.kind {
            'i' => println!("  {}", entry.label),
            'h' => {
                let url_prefix = "URL:";
                let mut url = entry.locator.as_str();

                if entry.locator.starts_with(url_prefix) {
                    url = &url[url_prefix.len()..];
                }

                println!("  [{}]({})", entry.label, url);
            }
            _ => println!(
                "{kind} [{label}]({host}:{port}{locator})",
                kind = entry.kind,
                label = entry.label,
                host = entry.host,
                port = entry.port,
                locator = entry.locator.as_str(),
            ),
        };

        Ok(())
    }).await
}