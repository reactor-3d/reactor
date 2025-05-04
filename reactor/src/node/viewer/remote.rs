use egui_snarl::{InPin, NodeId, OutPinId, Snarl};
use reactor_types::{Color, Float, Vector, Vector4};

use crate::node::{Node, Noded};

pub fn number<'a>(pin: &InPin, name: &str, snarl: &'a Snarl<Node>) -> Option<Float> {
    value(pin, name, |remote| match &snarl[remote.node] {
        Node::Number(number) => Ok(number.value()),
        node => Err(node),
    })
}

pub fn vector(pin: &InPin, name: &str, snarl: &Snarl<Node>) -> Option<Vector> {
    value(pin, name, |remote| match &snarl[remote.node] {
        Node::Number(number) => Ok(Vector::from_scalar(number.value())),
        Node::Vector(vector) => Ok(vector.value()),
        Node::Color(color) => {
            let color = color.value().to_normalized_gamma_f32();
            Ok(Vector::Dim4(Vector4::new(
                color[0] as _,
                color[1] as _,
                color[2] as _,
                color[3] as _,
            )))
        },
        node => Err(node),
    })
}

pub fn color(pin: &InPin, name: &str, snarl: &Snarl<Node>) -> Option<Color> {
    value(pin, name, |remote| match &snarl[remote.node] {
        Node::Number(number) => Ok(Color::from_gray((number.value() * 255.0).round() as u8)),
        Node::Color(color) => Ok(color.value()),
        Node::Vector(vector) => match vector.value() {
            Vector::Dim2(vector) => {
                let mut color = Color::from_gray((vector.x * 255.0).round() as u8);
                color[4] = (vector.y * 255.0).round() as u8;
                Ok(color)
            },
            Vector::Dim3(vector) => Ok(Color::from_rgb(
                (vector.x * 255.0).round() as u8,
                (vector.y * 255.0).round() as u8,
                (vector.z * 255.0).round() as u8,
            )),
            Vector::Dim4(vector) => Ok(Color::from_rgba_premultiplied(
                (vector.x * 255.0).round() as u8,
                (vector.y * 255.0).round() as u8,
                (vector.z * 255.0).round() as u8,
                (vector.w * 255.0).round() as u8,
            )),
        },

        node => Err(node),
    })
}

pub fn node<'a>(
    pin: &InPin,
    name: &str,
    snarl: &'a Snarl<Node>,
    filter: impl FnOnce(&'a Node) -> bool,
) -> Option<NodeId> {
    value(pin, name, |remote| match &snarl[remote.node] {
        node if filter(node) => Ok(remote.node),
        node => Err(node),
    })
}

pub fn value<'a, T>(pin: &InPin, name: &str, filter: impl FnOnce(&OutPinId) -> Result<T, &'a Node>) -> Option<T> {
    match &*pin.remotes {
        [] => None,
        [remote] => match filter(remote) {
            Ok(value) => Some(value),
            Err(node) => unreachable!("{name} input not support connection with `{}`", node.name()),
        },
        _ => None,
    }
}
