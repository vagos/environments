#![allow(unused)]

use std::ops::Deref;
use std::path::Path;
use std::cmp;
use std::{sync::*, fs};

use parry3d::query::Ray;
use rlua::prelude::*;
use clingo::*;
use raylib::prelude::*;

use parry3d::*;

use crate::na::SVector;

#[derive(Default)]
struct Entity {
    name: String,
    id: String,
    pos: Vector3,
    aabb: AABB,
    model: Option<raylib::prelude::Model>,
    scale: f32,
}


#[derive(Debug)]
struct AABB {
    aabb: parry3d::bounding_volume::AABB
}

impl Deref for AABB {
    type Target = parry3d::bounding_volume::AABB;

    fn deref(&self) -> &Self::Target {
        &self.aabb
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            aabb: parry3d::bounding_volume::AABB::new(
                       parry3d::na::Point3::new(0.0, 0.0, 0.0),
                       parry3d::na::Point3::new(1.0, 1.0, 1.0)
                  )
        }
    }
}


impl Entity {
    fn new(name: String) -> Self {

        let id = format!("{}{}", name, rand::random::<i32>());

        Self {
            name,
            id,
            ..Default::default()

        }
    }
}

struct LanguageInterface {

    state: rlua::Lua

}

impl LanguageInterface {

    fn new() -> Self {
        Self { state: Lua::new() }
    }

    fn create_callbacks(&self, world: &World) {

        let lua = &self.state;

        let entities_to_add = Arc::clone(&world.entities_to_add);

        lua.context(|lua_ctx| {

            let globals = lua_ctx.globals();

            let add_function = lua_ctx.create_function(move |_, name: rlua::String| {

                    entities_to_add.lock().unwrap().push(name.to_str().unwrap().to_string());
                    Ok(())
            });

            globals.set("Add", add_function.unwrap()).unwrap();

        });
        
    }

    fn load_file(&self, filename: &Path) {
        let file_contents = fs::read_to_string(filename)
            .unwrap_or("".to_string());

        self.state.context(|ctx| {

            ctx
                .load(&file_contents)
                .set_name("Main Script").unwrap()
                .exec().unwrap_or_else(|error| {
                    println!("{}", error);
                });

        });

    }

}

struct World {
    entities_to_add : Arc<Mutex<Vec<String>>>,
    entities : Arc<Mutex<Vec<Entity>>>,

        

}

impl World {
    
    fn new() -> Self {
        Self { 
            entities_to_add: Arc::from(Mutex::from(Vec::new())),
            entities: Arc::from(Mutex::from(Vec::new()))
        }
    }

    fn add_entities(&self, renderer: &mut Renderer) { // TODO Send entity creation somewhere else
        for e_n in self.entities_to_add.lock().unwrap().iter() {

            let mut e = Entity::new(e_n.to_string()); 
            let e_model = renderer.load_model(e_n, Path::new("./assets/"));
            let e_aabb = Renderer::calculate_aabb(&e_model);

            e.model = Some(e_model);
            e.aabb = e_aabb;

            self.add_entity(e);
        }
    }

    fn add_entity(&self, e: Entity) {
        // calculate aabb
        self.entities.lock().unwrap().push(e);
    }

    fn place_objects(&self) {

        let mut ctl = control(vec![]).expect("Failed creating Clingo control!"); // Maybe turn this into a LanguageInterface instance.

        let program = fs::read_to_string(Path::new("do.asp")).unwrap();

        ctl.add("base", &[], &program);

        let mut fb = FactBase::new();

        for e in self.entities.lock().unwrap().iter_mut() {

            let id = Symbol::create_id(&e.id, true).unwrap();
            let name = Symbol::create_id(&e.name, true).unwrap();
            let object = Symbol::create_function("object", &vec![id], true).unwrap();
            let type_symbol = Symbol::create_function("type", &vec![id, name], true).unwrap();
            
            let aabb = &e.aabb;

            let s_x = (aabb.maxs.x - aabb.mins.x).max(1.0) as i32;
            let s_y = (aabb.maxs.y - aabb.mins.y).max(1.0) as i32;
            let s_z = (aabb.maxs.z - aabb.mins.z).max(1.0) as i32;

            let size = vec![    
                id,
                Symbol::create_number(s_x),
                Symbol::create_number(s_y),
                Symbol::create_number(s_z),
                ];
            
            println!("{} {} {} {}", name, s_x, s_y, s_z);

            let size = Symbol::create_function("size", &size, true).unwrap();

            fb.insert(&object);
            fb.insert(&size);
            fb.insert(&type_symbol);
        }

        ctl.add_facts(&fb);

        let part = Part::new("base", vec![]).unwrap();
        let parts = vec![part];
        ctl.ground(&parts)
            .expect("Failed to ground a logic program.");


        let mut handle = ctl
            .solve(SolveMode::YIELD, &[])
            .expect("Failed retrieving solve handle.");

        loop {
            handle.resume().expect("Failed resume on solve handle.");
            match handle.model() {
                Ok(Some(model)) => {

                    for s in model.symbols(ShowType::SHOWN).unwrap() {
                        println!("Solved: {}", s);
                        match s.arguments().unwrap()[..] {
                            [name, x, y, z] => { 
                                for e in Arc::clone(&self.entities).lock().unwrap().iter_mut() {
                                    if name.to_string() == e.id {
                                        e.pos = Vector3::new(
                                            x.number().unwrap() as f32,
                                            z.number().unwrap() as f32,
                                            y.number().unwrap() as f32,
                                        );
                                    }
                                }
                            }
                            _ => todo!{}
                        }
                    }

                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    panic!("Error: {}", e);
                }
            }
        }

        // close the solve handle
        handle.close().expect("Failed to close solve handle.");


    }
}

struct RaylibContext {
    rl: RaylibHandle,
    thread: RaylibThread
}

struct Renderer {
    context: RaylibContext,
    camera: Camera3D

}

impl Renderer {

    fn new() -> Renderer {

       let (mut rl, thread) = raylib::init()
           .size(1000, 1000)
           .title("Main")
           .build();

       let context = RaylibContext { rl, thread };

       let mut camera = Camera3D::perspective(
                        Vector3::new(4.0, 2.0, 4.0),
                        Vector3::new(0.0, 1.8, 0.0),
                        Vector3::new(0.0, 1.0, 0.0),
                        60.0,
                    );

       let mut s = Self {
            camera,
            context
       };

       let mut rl = &mut s.context.rl;

       rl.set_camera_mode(&camera, CameraMode::CAMERA_FIRST_PERSON);
       rl.set_target_fps(60);

       s
    }

    fn load_model(&mut self, name: &String, path: &Path) -> raylib::prelude::Model {

       let obj_path = path.join(format!("{}.obj", name));
       let mtl_path = path;

       let mut model = self.context.rl.load_model(&self.context.thread, obj_path.to_str().unwrap());

       match model {

            Ok(o) => { return o; }
            Err(e) => { println!("{}", e); panic!(); }
       }
    }

    fn calculate_aabb(model: &raylib::prelude::Model) -> AABB {

       let mut min_p = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
       let mut max_p = Vector3::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY);

       for v in model.meshes().get(0).unwrap().vertices() {
               min_p.x = v.x.min(min_p.x);
               min_p.y = v.y.min(min_p.y);
               min_p.z = v.z.min(min_p.z);

               max_p.x = v.x.max(max_p.x);
               max_p.y = v.y.max(max_p.y);
               max_p.z = v.z.max(max_p.z);
       }
       
       AABB { 
           aabb: parry3d::bounding_volume::AABB::new( 
                        parry3d::na::Point3::new(min_p.x, min_p.y, min_p.z),
                        parry3d::na::Point3::new(max_p.x, max_p.y, max_p.z),
                       )
        }
    }

    fn update(&mut self, world: &World) -> bool {

        let mut rl = &mut self.context.rl;
        let window_close = rl.window_should_close();

        rl.update_camera(&mut self.camera);

        let mut d = rl.begin_drawing(&self.context.thread);

        d.clear_background(Color::WHITE);
        {
            let mut d = d.begin_mode3D(self.camera);

            for e in Arc::clone(&world.entities).lock().unwrap().iter() {
                d.draw_model(e.model.as_ref().unwrap(), e.pos, 1.0, Color::WHITE);
            }

            d.draw_plane(
                Vector3::new(0.0, 0.0, 0.0),
                Vector2::new(32.0, 32.0),
                Color::LIGHTGRAY,
            );
        }

        return !window_close;
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    
    let lua_interface = LanguageInterface::new();
    let world = World::new();
    let mut renderer = Renderer::new();

    lua_interface.create_callbacks(&world);
    lua_interface.load_file(Path::new("main.lua"));

    println!("{:?}", world.entities_to_add.lock().unwrap());

    world.add_entities(&mut renderer);
    world.place_objects();

    loop {
        if !renderer.update(&world) { break; }
    }
}
