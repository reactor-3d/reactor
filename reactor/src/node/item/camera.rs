use egui::{InputState, Key, Pos2, Ui, Vec2};
use egui_snarl::ui::PinInfo;
use egui_snarl::{InPin, NodeId, OutPin, Snarl};
use reactor_derives::Noded;
use reactor_types::angle::convert_angle_down;
use reactor_types::vector::convert_vector3_down;
use reactor_types::{Angle, Float, Matrix3, NodePin, Vector, Vector3};
use serde::{Deserialize, Serialize};

use crate::node::message::{MessageHandling, SelfNodeMut};
use crate::node::viewer::ui::{input, output};
use crate::node::{Node, NodeFlags, Noded};

#[derive(Clone, Serialize, Deserialize, Noded)]
pub struct CameraNode {
    pub position: NodePin<Vector>,
    pub yaw: NodePin<Angle>,
    pub pitch: NodePin<Angle>,
    /// vfov angle must be between 0..=90 degrees.
    pub vfov: NodePin<Angle>,
    /// Aperture must be between 0..=1.
    pub aperture: NodePin<Float>,
    /// Focus distance must be a positive number.
    pub focus_distance: NodePin<Float>,

    previous_mouse_pos: Option<Pos2>,
}

impl Default for CameraNode {
    fn default() -> Self {
        let look_from = Vector3::new(-10.0, 2.0, -4.0);
        let look_at = Vector3::new(0.0, 1.0, 0.0);
        let focus_distance = (look_at - look_from).magnitude();

        Self {
            position: NodePin::new(Vector::Dim3(look_from)),
            yaw: NodePin::new(Angle::degrees(25.0)),
            pitch: NodePin::new(Angle::degrees(-10.0)),
            vfov: NodePin::new(Angle::degrees(30.0)),
            aperture: NodePin::new(0.8),
            focus_distance: NodePin::new(focus_distance),

            previous_mouse_pos: None,
        }
    }
}

impl CameraNode {
    pub const NAME: &str = "Camera";
    pub const INPUTS: [u64; 6] = [
        NodeFlags::TYPICAL_VECTOR_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
        NodeFlags::TYPICAL_NUMBER_INPUT.bits(),
    ];
    pub const OUTPUTS: [u64; 1] = [NodeFlags::CAMERA.bits()];

    pub fn to_xrays_camera(&self) -> xrays::Camera {
        let orientation = self.orientation();

        xrays::Camera {
            eye_pos: convert_vector3_down(&self.position.get().as_dim3()),
            eye_dir: convert_vector3_down(&orientation.forward),
            up: convert_vector3_down(&orientation.up),
            vfov: convert_angle_down(self.vfov.get()),
            aperture: self.aperture.get() as _,
            focus_distance: self.focus_distance.get() as _,
        }
    }
}

impl MessageHandling for CameraNode {
    fn handle_display_input(self_node: SelfNodeMut, pin: &InPin, ui: &mut Ui) -> Option<PinInfo> {
        match pin.id.input {
            0 => Some(input::display_vector_field(ui, pin, self_node, "Position", |node| {
                &mut node.as_camera_mut().position
            })),
            1 => Some(input::display_as_number_field(ui, pin, self_node, "Yaw", |node| {
                &mut node.as_camera_mut().yaw
            })),
            2 => Some(input::display_as_number_field(ui, pin, self_node, "Pitch", |node| {
                &mut node.as_camera_mut().pitch
            })),
            3 => Some(input::display_as_number_field(ui, pin, self_node, "VFOV", |node| {
                &mut node.as_camera_mut().vfov
            })),
            4 => Some(input::display_number_field(ui, pin, self_node, "Aperture", |node| {
                &mut node.as_camera_mut().aperture
            })),
            5 => Some(input::display_number_field(
                ui,
                pin,
                self_node,
                "Focus Distance",
                |node| &mut node.as_camera_mut().focus_distance,
            )),
            _ => None,
        }
    }

    fn handle_display_output(_self_node: SelfNodeMut, _pin: &OutPin, _ui: &mut Ui) -> Option<PinInfo> {
        Some(output::empty_view())
    }
}

#[derive(Clone, Debug)]
pub struct Orientation {
    pub forward: Vector3,
    pub right: Vector3,
    pub up: Vector3,
}

impl CameraNode {
    pub fn orientation(&self) -> Orientation {
        let forward = Vector3::new(
            self.yaw.as_ref().as_radians().cos() * self.pitch.as_ref().as_radians().cos(),
            self.pitch.as_ref().as_radians().sin(),
            self.yaw.as_ref().as_radians().sin() * self.pitch.as_ref().as_radians().cos(),
        )
        .normalize();

        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let right = forward.cross(&world_up);
        let up = right.cross(&forward);

        Orientation { forward, right, up }
    }

    pub fn after_events(&mut self, input_state: &InputState) {
        let translation_scale = 2.0 * input_state.stable_dt as f64;
        let look_pressed = input_state.pointer.secondary_down();
        let forward_pressed = input_state.key_pressed(Key::W);
        let backward_pressed = input_state.key_pressed(Key::S);
        let left_pressed = input_state.key_pressed(Key::A);
        let right_pressed = input_state.key_pressed(Key::D);
        let down_pressed = input_state.key_pressed(Key::Q);
        let up_pressed = input_state.key_pressed(Key::E);
        let mouse_pos = input_state.pointer.latest_pos().unwrap_or_default();
        let viewport_size = input_state
            .viewport()
            .inner_rect
            .map(|rect| rect.size())
            .unwrap_or_default();

        if look_pressed {
            if let Some(prev_mouse_pos) = self.previous_mouse_pos {
                let orientation = self.orientation();
                let c1 = orientation.right;
                let c2 = orientation.forward;
                let c3 = c1.cross(&c2).normalize();
                let from_local = Matrix3::new(c1.x, c2.x, c3.x, c1.y, c2.y, c3.y, c1.z, c2.z, c3.z);
                let to_local = from_local.try_inverse().expect("Could not invert matrix");

                // Perform cartesian to spherical coordinate conversion in camera-local space,
                // where z points straight into the screen. That way there is no need to worry
                // about which quadrant of the sphere we are in for the conversion.
                let current_dir = to_local * self.generate_ray_dir(mouse_pos, viewport_size);
                let previous_dir = to_local * self.generate_ray_dir(prev_mouse_pos, viewport_size);

                let x1 = current_dir.x;
                let y1 = current_dir.y;
                let z1 = current_dir.z;

                let x2 = previous_dir.x;
                let y2 = previous_dir.y;
                let z2 = previous_dir.z;

                let p1 = z1.acos();
                let p2 = z2.acos();

                let a1 = y1.signum() * (x1 / (x1 * x1 + y1 * y1).sqrt()).acos();
                let a2 = y2.signum() * (x2 / (x2 * x2 + y2 * y2).sqrt()).acos();

                *self.yaw.as_mut() = self.yaw.get() + Angle::radians(a1 - a2);
                *self.pitch.as_mut() =
                    (self.pitch.get() + Angle::radians(p1 - p2)).clamp(Angle::degrees(-89.0), Angle::degrees(89.0));
            }
        }

        {
            let v = |b| if b { 1.0 } else { 0.0 };
            let translation = Vector3::new(
                translation_scale * (v(right_pressed) - v(left_pressed)),
                translation_scale * (v(up_pressed) - v(down_pressed)),
                translation_scale * (v(forward_pressed) - v(backward_pressed)),
            );

            let orientation = self.orientation();
            *self.position.as_mut() = Vector::Dim3(
                self.position.get().as_dim3()
                    + orientation.right * translation.x
                    + orientation.up * translation.y
                    + orientation.forward * translation.z,
            );
        }

        self.previous_mouse_pos = Some(mouse_pos);
    }

    fn generate_ray_dir(&self, mouse_pos: Pos2, viewport_size: Vec2) -> Vector3 {
        let position = self.position.get().as_dim3();
        let focus_distance = self.focus_distance.get();
        let aspect_ratio = viewport_size.x as f64 / viewport_size.y as f64;
        let half_height = focus_distance * (0.5 * self.vfov.get().as_radians()).tan();
        let half_width = aspect_ratio * half_height;

        let x = mouse_pos.x as f64 / (viewport_size.x as f64);
        let y = mouse_pos.y as f64 / (viewport_size.y as f64);

        let orientation = self.orientation();

        let point_on_plane = position
            + focus_distance * orientation.forward
            + (2.0 * x - 1.0) * half_width * orientation.right
            + (1.0 - 2.0 * y) * half_height * orientation.up;

        (point_on_plane - position).normalize()
    }
}

pub fn camera_node_by_id(camera_id: NodeId, snarl: &Snarl<Node>) -> Option<&CameraNode> {
    snarl.get_node(camera_id).and_then(Node::camera_ref)
}
