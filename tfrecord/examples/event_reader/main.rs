use anyhow::Result;
use csv::Writer;
use indexmap::IndexSet;
use serde::Serialize;
use std::path::PathBuf;
use structopt::StructOpt;
use tfrecord::{
    protobuf::{event::What, summary::value::Value::SimpleValue, Summary},
    EventIter,
};

#[derive(StructOpt)]
struct Args {
    #[structopt(long, short, default_value = "./sample_event")]
    event: String,

    #[structopt(long, short)]
    tags: Option<Vec<String>>,

    #[structopt(long, short, default_value = "./output")]
    output: PathBuf,
}

struct TagData {
    step: i64,
    tag: String,
    value: f32,
}

#[derive(Serialize, Debug)]
struct Data {
    step: i64,
    value: f32,
}

impl From<&TagData> for Data {
    fn from(data: &TagData) -> Self {
        Self {
            step: data.step,
            value: data.value,
        }
    }
}

fn main() -> Result<()> {
    let Args {
        event: event_file,
        tags: query_tags,
        output: output_dir,
    } = Args::from_args();
    let reader = EventIter::open(&event_file, Default::default())?;
    std::fs::create_dir_all(&output_dir)?;

    let history: Vec<TagData> = reader
        .into_iter()
        .map(|result| -> Result<_> {
            let event = result?;
            let tag = match event.what {
                Some(What::Summary(Summary { value })) => {
                    let val = &value[0];
                    match val.value {
                        Some(SimpleValue(value)) => Some(TagData {
                            step: event.step,
                            tag: val.tag.clone(),
                            value,
                        }),
                        _ => None,
                    }
                }
                _ => None,
            };
            Ok(tag)
        })
        .filter_map(|result| result.transpose())
        .collect::<Result<_>>()?;

    let tag_list = {
        let mut tags: IndexSet<_> = history.iter().map(|tag_data| &tag_data.tag).collect();
        tags.sort();
        tags
    };

    if let Some(query_tags) = query_tags {
        query_tags.iter().try_for_each(|tag| -> Result<_> {
            if !tag_list.contains(&tag) {
                eprintln!(
                    "Warning: The tag {:?} is not found in valid tags {:#?}",
                    tag, tag_list
                );
                return Ok(());
            }

            let output_file = output_dir.join(format!("{}.csv", tag.replace("/", "-")));
            let mut wtr = Writer::from_path(&output_file)?;
            history
                .iter()
                .filter(|data| &data.tag == tag)
                .try_for_each(|data| wtr.serialize(Data::from(data)))?;
            wtr.flush()?;
            println!("The tag data is saved to {}", output_file.display());
            Ok(())
        })?;
    } else {
        println!("The tags inside this event file are {:#?}", tag_list);
        println!("Please specify the tags to be extracted.");
    }

    Ok(())
}
