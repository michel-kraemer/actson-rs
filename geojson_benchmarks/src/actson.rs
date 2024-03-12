use std::{fs::File, io::BufReader, path::PathBuf};

use actson::{
    feeder::{BufReaderJsonFeeder, PushJsonFeeder},
    tokio::AsyncBufReaderJsonFeeder,
    JsonEvent, JsonParser,
};
use anyhow::{Ok, Result};
use tokio::{io::AsyncReadExt, sync::mpsc};

pub async fn bench_bufreader(path: &PathBuf) -> Result<u64> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let reader = BufReader::new(file);

    let feeder = BufReaderJsonFeeder::new(reader);
    let mut parser = JsonParser::new(feeder);
    while let Some(event) = parser.next_event()? {
        match event {
            JsonEvent::NeedMoreInput => parser.feeder.fill_buf()?,

            // make sure all values are parsed
            JsonEvent::FieldName => _ = parser.current_str(),
            JsonEvent::ValueString => _ = parser.current_str(),
            JsonEvent::ValueInt => _ = parser.current_int::<i64>(),
            JsonEvent::ValueFloat => _ = parser.current_float(),

            _ => {}
        }
    }

    Ok(len)
}

pub async fn bench_tokio(path: &PathBuf) -> Result<u64> {
    let file = tokio::fs::File::open(path).await?;
    let len = file.metadata().await?.len();
    let reader = tokio::io::BufReader::new(file);

    let feeder = AsyncBufReaderJsonFeeder::new(reader);
    let mut parser = JsonParser::new(feeder);
    while let Some(event) = parser.next_event()? {
        match event {
            JsonEvent::NeedMoreInput => parser.feeder.fill_buf().await?,

            // make sure all values are parsed
            JsonEvent::FieldName => _ = parser.current_str(),
            JsonEvent::ValueString => _ = parser.current_str(),
            JsonEvent::ValueInt => _ = parser.current_int::<i64>(),
            JsonEvent::ValueFloat => _ = parser.current_float(),

            _ => {} // do something useful with the event
        }
    }

    Ok(len)
}

pub async fn tokio_twotasks(path: &PathBuf) -> Result<u64> {
    let (tx, mut rx) = mpsc::channel(1);

    let mut file = tokio::fs::File::open(path).await?;
    let len = file.metadata().await?.len();

    let reader_task = tokio::spawn(async move {
        loop {
            let mut buf = vec![0; 65 * 1024];
            let r = file.read(&mut buf).await?;
            if r == 0 {
                break;
            }
            buf.truncate(r);
            tx.send(buf).await?;
        }

        Ok(())
    });

    let parser_task = tokio::spawn(async move {
        let feeder = PushJsonFeeder::new();
        let mut parser = JsonParser::new(feeder);
        let mut i = 0;
        let mut buf = Vec::new();
        while let Some(event) = parser.next_event()? {
            match event {
                JsonEvent::NeedMoreInput => {
                    i += parser.feeder.push_bytes(&buf[i..]);
                    if i == buf.len() {
                        if let Some(b) = rx.recv().await {
                            buf = b;
                            i = 0;
                        } else {
                            parser.feeder.done();
                        }
                    }
                }

                // make sure all values are parsed
                JsonEvent::FieldName => _ = parser.current_str(),
                JsonEvent::ValueString => _ = parser.current_str(),
                JsonEvent::ValueInt => _ = parser.current_int::<i64>(),
                JsonEvent::ValueFloat => _ = parser.current_float(),

                _ => {} // do something useful with the event
            }
        }

        Ok(())
    });

    reader_task.await??;
    parser_task.await??;

    Ok(len)
}
