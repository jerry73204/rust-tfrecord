use anyhow::Result;
use clap::Clap;
use csv::Writer;
use std::collections::HashSet;
use tfrecord::{
    protos::{
        event::What,
        summary::value::Value::SimpleValue,
        Event,
    },
    RecordReader, RecordReaderInit,
};
use serde::Serialize;

#[derive(Clap)]
struct Args {
    #[clap(long, short, default_value = "./sample_event")]
    event: String,

    #[clap(long, short)]
    tags: Option<Vec<String>>,

    #[clap(long, short, default_value = "./output")]
    output: String,
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
    let args = Args::parse();
    let reader: RecordReader<Event, _> = RecordReaderInit::default().open(&args.event)?;
    std::fs::create_dir_all(&args.output)?;

    let history = reader
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|event| match event.what {
            Some(What::Summary(summary)) => {
                let val = &summary.value[0];
                match val.value {
                    Some(SimpleValue(value)) => Some(TagData {
                        step: event.step,
                        tag: val.tag.clone(),
                        value: value,
                    }),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect::<Vec<TagData>>();

    let tag_list = {
        let mut tags = HashSet::new();
        history.iter().for_each(|x| { tags.insert(x.tag.clone()); });
        let mut vec = tags.into_iter().collect::<Vec<String>>();
        vec.sort();
        vec
    };

    if let Some(tags) = args.tags {
        for tag in tags {
            assert!(
                tag_list.contains(&tag),
                "\nThe tag {:?} not in valid tags {:#?}",
                tag,
                tag_list,
            );
            let file_name = format!(
                "{}/{}.csv",
                &args.output,
                tag.replace("/", "-"),
            );
            let mut wtr = Writer::from_path(&file_name)?;
            history.iter().for_each(|x| {
                if x.tag == tag {
                    wtr.serialize(Data::from(x)).unwrap();
                }
            });
            wtr.flush()?;
            println!("The tag data has been saved as {}", &file_name);
        }
    } else {
        println!("The tags inside this event are {:#?}", &tag_list);
        println!("Please specify the tags to be extracted.");
    }
    Ok(())
}
