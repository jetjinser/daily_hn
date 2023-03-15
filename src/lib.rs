use rusted_firebase::{Firebase, Paramable, Requestable};
use schedule_flows::schedule_cron_job;
use serde_json::Value;
use slack_flows::send_message_to_channel;

#[no_mangle]
pub fn run() {
    schedule_cron_job(
        String::from("0 10 * * *"),
        String::from("cron_job_evoked"),
        callback,
    );
}

fn callback(_body: Vec<u8>) {
    let firebase =
        Firebase::new("https://hacker-news.firebaseio.com/v0/").expect("error init Firebase");

    let cons = firebase
        .at("topstories")
        .limit_to_first(7)
        .order_by("\"$key\"");

    let resp = cons.get::<Vec<usize>>();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();

    if let Ok(rt) = rt {
        let item_ids = rt.block_on(resp).unwrap_or(vec![]);
        let items = item_ids
            .iter()
            .map(|item_id| {
                let cons = firebase.at("item").at(&item_id.to_string());
                let resp = cons.get::<Value>();

                let value = rt.block_on(resp).unwrap_or(Value::Null);

                let by = value
                    .get("by")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("Unknown `by`");
                let title = value
                    .get("title")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("Unknown `title`");
                let url = value
                    .get("url")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .unwrap_or("Unknown `url`");

                format!("#{}\n<{}|source> by {}", title, url, by)
            })
            .collect::<Vec<_>>()
            .join("\n");

        send_message_to_channel("ham-5b68442", "general", items);
    }
}
