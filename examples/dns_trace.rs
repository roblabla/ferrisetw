use ferrisetw::native::etw_types::EventRecord;
use ferrisetw::parser::{Parser, TryParse};
use ferrisetw::provider::*;
use ferrisetw::schema::SchemaLocator;
use ferrisetw::property::*;
use ferrisetw::trace::*;
use windows::Guid;
use std::time::Duration;

fn main() {
    let image_load_callback =
        |record: EventRecord, schema_locator: &mut SchemaLocator| match schema_locator
            .event_schema(record)
        {
            Ok(schema) => {
                match schema.event_id() {
                    3006 => {
                        let mut parser = Parser::create(&schema);
                        let pid = schema.process_id();
                        let query_name: String = parser.try_parse("QueryName").unwrap();
                        let query_type: u32 = parser.try_parse("QueryType").unwrap();
                        println!("{} {} {}", pid, query_type, query_name);
                    },
                    _ => (),
                }
            }
            Err(err) => println!("Error {:?}", err),
        };

    let provider = Provider::new()
        .by_guid("1C95126E-7EEA-49A9-A3FE-A378B03DDB4D")
        .add_callback(image_load_callback)
        .build()
        .unwrap();

    let mut trace = UserTrace::new()
        .enable(provider)
        .start()
        .unwrap();

    std::thread::sleep(Duration::new(20, 0));
    trace.stop();
}
