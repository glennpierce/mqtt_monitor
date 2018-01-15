use error_chain;
use std::io;
use serde_json;
use csv;
use chrono;
use std::ops;
use postgres;
use std::sync::mpsc::SendError;
use element::{ElementPad, ElementStreamFormat};
use iron::IronError;

error_chain!{

    errors {

      InvalidParameter(message: String) {
             description("InvalidParameter")
             display("InvalidParameter: '{}'", message)
        }

        ElementPadNotInput(element_name: String, pad_name: String) {
             description("Element pad is not an input")
             display("Element pad is not an input: '{} - {}'", element_name, pad_name)
        }

        ElementPadNotOutput(element_name: String, pad_name: String) {
             description("Element pad is not an output")
             display("Element pad is not an output: '{} - {}'", element_name, pad_name)
        }

        ElementDataSendError(data: ElementStreamFormat, err: String) {
              description("Element failed to send data")
              display("Element failed to send data: {}. Err: {}", data, err)
        }

        ElementDataReceiveError(err: String) {
              description("Element failled to receive data")
              display("Element failed to receive data: {}", err)
        }

        ElementPadIncompatibleFormat(pad1_name: String, pad2_name: String) {
             description("Element pads have incompatible formats")
             display("Element pads have incompatible formats: '{} - {}'", pad1_name, pad2_name)
        }

        ElementNoSuchProperty(property_name: String) {
              description("No such property")
              display("Property {} does not exist", property_name)
        }

        ElementWrongType(msg: String) {
              description("Element wrong type")
              display("Element wrong type: {}", msg)
        }

        PipelineNotExist(pipeline_name: String) {
              description("Pipeline does not exist with that name")
              display("Pipeline {} does not exist", pipeline_name)
        }

        PipelineNoSink(pipeline_name: String) {
              description("Pipeline has no sink element")
              display("Pipeline {} has no sink element", pipeline_name)
        }

        PipelinetDataFormatNotSupported(pipeline_name: String, pad: ElementPad) {
             description("Element data format is not support")
             display("Element data format is not supported: 'pipeline_name {} - pad {:?}'", pipeline_name, pad)
        }

        PipelineNoSource(pipeline_name: String) {
              description("Pipeline has no source element")
              display("Pipeline {} has no source element", pipeline_name)
        }

        PipelineDuplicateConnection(pipeline_name: String, connection: String) {
              description("Pipeline element and channel already connected")
              display("Pipeline {} element and channel {} already connected", pipeline_name, connection)
        }

        PipelineMoreThanOneSink(pipeline_name: String) {
              description("Pipeline has more than one sink element")
              display("Pipeline {} has more than one sink element", pipeline_name)
        }

        PipelineNoConnections(pipeline_name: String) {
              description("Pipeline has no connections")
              display("Pipeline {} has no connections", pipeline_name)
        }

        PipelineRunError(pipeline_name: String) {
              description("Pipeline error when running")
              display("Pipeline {} error when running", pipeline_name)
        }

        PipelineFileFormatCorrupt(pipeline_name: String, filepath: String) {
              description("Pipeline file format corrupt")
              display("Pipeline: {} filepath: {} file format corrupt", pipeline_name, filepath)
        }

        UnknownError(message: String) {
              description("Unknown Error")
              display("Unknown Error: {}", message)
        }
    }

    foreign_links {
        Io(io::Error);
        Json(serde_json::Error) #[doc = "An error happened while serializing JSON"];
        Csv(csv::Error);
        DateTimeError(chrono::ParseError);
        PostgresError(postgres::error::Error);
        ChannelError(SendError<ElementStreamFormat>);
        //WebError(IronError);
        //CarrierError(ops::Carrier);
    }

}
