#[macro_use] extern crate glium;
extern crate ocl;
extern crate cgl;

mod particles;
mod point;

use particles::Particles;
use point::Point;

use glium::{DisplayBuild, Surface, VertexBuffer, IndexBuffer, Program, GlObject};
use glium::index::{NoIndices, PrimitiveType};
use glium::glutin::{Event, ElementState, VirtualKeyCode};

const VERTEX_SRC: &'static str = include_str!("shaders/default.vert");
const FRAGMENT_SRC: &'static str = include_str!("shaders/default.frag");

use ocl::{core, ProQue, Buffer, Device, Platform, Context, Program as ProgramCl};
use ocl::core::{ContextProperties, DeviceType, DeviceInfo};
use ocl::builders::DeviceSpecifier;
use ocl::cl_h::CL_DEVICE_TYPE_GPU;

const KERNEL_SRC: &'static str = include_str!("kernels/particle_management.cl");

use cgl::{CGLGetCurrentContext, CGLGetShareGroup};

#[derive(Copy, Clone)]
struct Particle {
    position: [f32; 2],
    color: [f32; 3], // to delete
}

implement_vertex!(Particle, position, color);

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(1024, 768)
        .with_title(String::from("Hello world"))
        .build_glium()
        .unwrap();

    let vertex_buffer = VertexBuffer::dynamic(&display,
            &[
                Particle { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
                Particle { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0] },
                Particle { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0] },
            ]).unwrap();
    // let indices = NoIndices(PrimitiveType::Points);
    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap();
    let program = Program::from_source(&display, VERTEX_SRC, FRAGMENT_SRC, None).unwrap();
    let uniforms = uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ]
    };

    let cgl_current_ctx = unsafe { CGLGetCurrentContext() };
    let cgl_share_grp = unsafe { CGLGetShareGroup(cgl_current_ctx) };
    let properties = ContextProperties::new().cgl_sharegroup(cgl_share_grp);

    let platform_cl = Platform::default();
    let device_type_cl = DeviceType::from_bits_truncate(CL_DEVICE_TYPE_GPU);
    let devices_cl = Device::list(&platform_cl, Some(device_type_cl));
    let device_cl = devices_cl.first().expect("No device with specified types found.");

    println!("device: {:?}", device_cl.info(DeviceInfo::Name));

    let context_cl = Context::builder()
                    .properties(properties)
                    .devices(DeviceSpecifier::Single(*device_cl))
                    .build().unwrap();

    let program_builder_cl = ProgramCl::builder().src(KERNEL_SRC);
    let pq_cl = ProQue::builder()
                .context(context_cl)
                .prog_bldr(program_builder_cl)
                .device(device_cl)
                .dims([5 * 3])
                .build().unwrap();

    let vertex_buffer_cl: Buffer<f32> = Buffer::from_gl_buffer(&pq_cl,
                                            Some(core::MEM_READ_WRITE),
                                            [5 * 3],
                                            vertex_buffer.get_id()
                                        ).unwrap();

    let mut local_vector = vec![0.0f32; 5 * 3];

    // Acquire buffer
    vertex_buffer_cl.cmd().gl_acquire().enq().unwrap();

    vertex_buffer_cl.read(&mut local_vector).enq().unwrap();
    println!("before: {:?}", local_vector);

    // Enqueue kernel:
    let kern = pq_cl.create_kernel("add_to_each").unwrap();
    println!("Kernel global work size: {:?}", kern.get_gws());
    kern.arg_buf(&vertex_buffer_cl)
        .arg_scl(0.2f32)
        .enq().unwrap();

    let kern2 = pq_cl.create_kernel("add_to_each").unwrap();
    kern2.arg_buf(&vertex_buffer_cl)
        .arg_scl(0.2f32)
        .enq().unwrap();

    vertex_buffer_cl.read(&mut local_vector).enq().unwrap();
    println!("after: {:?}", local_vector);

    // Release buffer
    vertex_buffer_cl.cmd().gl_release().enq().unwrap();

    let mut particles = Particles::new(&display, pq_cl, 1_000_000);
    particles.init_sphere();

    let grav_point = Point::new(0.0, 0.0, 0.0);
    particles.update(grav_point);

    for event in display.wait_events() {
        let mut frame = display.draw();
        frame.clear_color(0.17, 0.17, 0.17, 1.0);
        frame.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        frame.finish().unwrap();

        // println!("{:?}", event);

        match event {
            Event::Closed
            | Event::KeyboardInput(ElementState::Released, _, Some(VirtualKeyCode::Escape)) => break,
            _ => ()
        }
    }
}
