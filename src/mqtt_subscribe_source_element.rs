use std::any::Any;
use std::io;
use std::fmt;
use std::fmt::Debug;
use std::io::Write;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard, LockResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use std::{thread, time};
use thread_control::{Flag, Control};
use rumqtt::{MqttOptions, MqttClient, MqttCallback, Message, QoS};

use serde::{Serialize};
use serde_json;

use errors::*;
use element::*;
use pipeline::{PipelineElementData};

#[derive(Clone)]
pub struct MqttSubscribeSourceElement {
    buffer: ElementBuffer,
    output_pad1: ElementPad,
    host: ElementProperty,
    port: ElementProperty,
    user: ElementProperty,
    password: ElementProperty,
    topic: ElementProperty,
}

impl Default for MqttSubscribeSourceElement {
  fn default () -> MqttSubscribeSourceElement {
        MqttSubscribeSourceElement {buffer : ElementBuffer::new(),

                         output_pad1: ElementPad::new(
                                          ElementChannelType::CHANNEL1,
                                          "mqtt_subscribe_element_output1".to_string(),
                                          ElementPadType::OUTPUT,
                                          vec![STRING_MESSAGE],
                                          "Watches for any events from the mqtt broker"),

                         host : ElementProperty::new("host",
                                                            "",
                                                            ElementPropertyType::STRING,
                                                            "Host of mqtt server"
                                                            ),

                         port : ElementProperty::new("port",
                                                                "21",
                                                                ElementPropertyType::STRING,
                                                                "Port of mqtt server"
                                                                ),

                         user : ElementProperty::new("username",
                                                                "",
                                                                ElementPropertyType::STRING,
                                                                "Username of mqtt server"
                                                                ),

                         password : ElementProperty::new("password",
                                                                "",
                                                                ElementPropertyType::STRING,
                                                                "Port of mqtt server"
                                                                ),

                         topic : ElementProperty::new("topic",
                                                                "",
                                                                ElementPropertyType::STRING,
                                                                "Topic to subscribe"
                                                                ),
                        }
  }
}

impl MqttSubscribeSourceElement {
    pub fn new() -> Self {

        MqttSubscribeSourceElement {
             ..Default::default()
        }
    }
}

unsafe impl Sync for MqttSubscribeSourceElement {}

impl Element for MqttSubscribeSourceElement {

    fn get_element_name() -> &'static str {
        stringify!("MqttSubscribeSourceElement")
    }

    fn get_name(&self) -> String {
        "MqttSubscribeSourceElement".to_string()
    }

    fn box_clone(&self) -> Box<Element> {
        Box::new((*self).clone())
    }

    fn get_type(&self) -> ElementType {
        ElementType::SOURCE
    }

    fn run(&mut self,
            pipeline_name: String,
            pipeline_element_data: &mut PipelineElementData,
            flag: Flag,
            output1_pad: Option<ElementPad>,
            output2_pad: Option<ElementPad>,
            input1_pad: Option<ElementPad>,
            input2_pad: Option<ElementPad>,
            output1: WeakElementSenderOption,
            output2: WeakElementSenderOption,
            input1: WeakElementReceiverOption,
            input2: WeakElementReceiverOption) -> Result<()> {

        let mut buffer = &mut self.buffer;
        let buffer_arc = buffer.sender();
        let buffer_sender = buffer_arc.lock().unwrap();
        let client_id = pipeline_name.to_string();

        //let host = self.host.get_value();
        //let port = self.port.get_value();

        // Specify client connection options
        let client_options = MqttOptions::new()
                                        .set_keep_alive(5)
                                        .set_reconnect(3)
                                        .set_client_id("rumqtt-docs")
                                        .set_broker("192.168.1.7:1883");

        let msg_callback_fn = move |message : Message| {

            let payload = Arc::try_unwrap(message.payload).unwrap();
            let s = String::from_utf8_lossy(&payload);

            //println!("message --> {:?}", s);

            if output1.is_some() {

                let new_value = ElementStreamFormat::StringMessage(0, vec![s.to_string()]);

                output1.as_ref().unwrap().send(new_value);
            }

            //buffer.send(&buffer_sender, s.to_string());
        };

        let msg_callback = MqttCallback::new().on_message(msg_callback_fn);

        let mut request = MqttClient::start(client_options, Some(msg_callback)).expect("Coudn't start");

        let topics = vec![("test/basic", QoS::Level0)];
        request.subscribe(topics).expect("Subcription failure");


        // Loop just looks for end of data signal as mqtt lib has started another thread.
        loop {

            if !flag.is_alive() {
                break;
            }

            if input1.is_some() {
                match input1.as_ref().unwrap().recv() {

                    Ok(ElementStreamFormat::EndDataMessage(ref id)) => { 
                            info!(target: "overly-verbose-target", "mqtt subscribe element got end_data_message");
                            break;
                    },

                    _ => {},

                    Err(e) => {}
                }
            }
        }

        request.shutdown();

        Ok(())
    }

    fn set_property(&mut self, property_name : &str, property_value : &str) -> Result<()> {

        if property_name == "host" {
            self.host.set_value(property_value);
            return Ok(());
        }
        else if property_name == "port" {
            self.port.set_value(property_value);
            return Ok(());
        }
        else if property_name == "user" {
            self.user.set_value(property_value);
            return Ok(());
        }
        else if property_name == "password" {
            self.password.set_value(property_value);
            return Ok(());
        }
        else if property_name == "topic" {
            self.topic.set_value(property_value);
            return Ok(());
        }

        Ok(())
    }

    fn get_property(&self, property_name : &str) -> Option<String> {

        if property_name == "host" {
            return Some(self.host.get_value());
        }
        else if property_name == "port" {
            return Some(self.port.get_value());
        }
        else if property_name == "user" {
            return Some(self.user.get_value());
        }
        else if property_name == "password" {
            return Some(self.password.get_value());
        }
        else if property_name == "topic" {
            return Some(self.topic.get_value());
        }
    
        None
    }

    fn get_input_pads(&self) -> (Option<ElementPad>, Option<ElementPad>) {
        (None, None)
    }

    fn get_output_pads(&self) -> (Option<ElementPad>, Option<ElementPad>) {
        (Some(self.output_pad1.clone()), None)
    }

    fn get_buffer(&self) -> &ElementBuffer {
        &self.buffer
    }

    fn to_json(&self) -> serde_json::Value {
        json!({
            "name": "MqttSubscribeSourceElement",
            "element_type":  self.get_type(),
            "element_doc": "Creates a Mqtt Source subscribe element",
            "properties": {"host": self.host,
                           "port": self.port,
                           "user": self.user,
                           "password": self.password,
                           "topic": self.topic,
                          },
            "outputs": {"mqtt_subscribe_element_output1": self.output_pad1},
        })
    }

    fn from_json(&self, json_str : &str) -> Result<Box<Element>> {
        let value: serde_json::Value = serde_json::from_str(json_str)?;
        let mut obj = MqttSubscribeSourceElement::new();

        obj.host = serde_json::from_str(&value["properties"]["host"].to_string())?;
        obj.port = serde_json::from_str(&value["properties"]["port"].to_string())?;
        obj.user = serde_json::from_str(&value["properties"]["user"].to_string())?;
        obj.password = serde_json::from_str(&value["properties"]["password"].to_string())?;
        obj.topic = serde_json::from_str(&value["properties"]["topic"].to_string())?;

        Ok(Box::new(obj))
    }
}
