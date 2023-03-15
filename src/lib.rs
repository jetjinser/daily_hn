use http_req::request;
use schedule_flows::schedule_cron_job;
use select::{document::Document, predicate::Class};
use slack_flows::send_message_to_channel;

const URI: &str = "https://news.ycombinator.com/front";

#[no_mangle]
pub fn run() {
    schedule_cron_job(
        String::from("0 10 * * *"),
        String::from("cron_job_evoked"),
        callback,
    );
}

fn callback(_body: Vec<u8>) {
    let mut writer = Vec::new();
    let result = request::get(URI, &mut writer);

    if let Ok(resp) = result {
        if resp.status_code().is_success() {
            let s = String::from_utf8(writer).unwrap();
            let document = Document::from(s.as_str());

            let msg = document
                .find(Class("titleline"))
                .zip(document.find(Class("hnuser")))
                .filter_map(|(tl, user)| {
                    tl.first_child().zip(tl.last_child()).map(|(a, span)| {
                        let title = a.text();
                        let url = a.attr("href").unwrap_or_default();
                        let author = user.text();
                        let site = span
                            .find(Class("sitestr"))
                            .next()
                            .map(|span| span.text())
                            .unwrap_or("source".to_string());

                        format!("- *{title}*\n<{url}|/{site}> by {author}\n")
                    })
                })
                .collect::<String>();
            send_message_to_channel("ham-5b68442", "general", msg);
        }
    }
}
