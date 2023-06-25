use egui::{Pos2, Rect, Vec2, Stroke, Color32};
use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
    time::Duration, fs::FileType,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use web_sys::{Url, Window};

use image::{RgbImage, Rgb, DynamicImage};
use image::imageops::FilterType;

const CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
enum EditOption {
    PlayGame,
    EditMap(GameObject),
    Delete,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
enum GameObject {
    Hole(Point),
    Wall { a: Point, b: Point },
    Start(Point),
    Height { a: Point, b: Point, height: i32 },
}

impl GameObject {
    fn symbol(&self) -> String {
        match &self {
            GameObject::Hole(_) => "H".to_string(),
            GameObject::Wall { .. } => "W".to_string(),
            GameObject::Start(_) => "S".to_string(),
            GameObject::Height { .. } => "H".to_string(),
        }
    }

    fn point(&self) -> Option<Point> {
        match &self {
            GameObject::Hole(p) => Some(*p),
            GameObject::Wall { .. } => None,
            GameObject::Start(p) => Some(*p),
            GameObject::Height { .. } => None,
        }
    }

    fn get_points(&self) -> Vec<Point> {
        match &self {
            GameObject::Hole(p) => vec![*p],
            GameObject::Wall { a, b } => vec![*a, *b],
            GameObject::Start(p) => vec![*p],
            GameObject::Height { a, b, .. } => vec![*a, *b],
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(default)]
struct Point {
    x: i32,
    y: i32,
}

impl Default for Point {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }

}


impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
struct GolfBall {
    pos: Pos,
    vel: Pos,
    shoot: bool,
}

impl Default for GolfBall {
    fn default() -> Self {
        Self {
            pos: Pos::default(),
            vel: Pos::default(),
            shoot: false,
        }
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl GolfBall {
    fn update_pos(&mut self, map: &GolfMap, delta: f32) {
        let mut new_pos = self.pos.clone();
        self.vel.x *= 0.98;
        self.vel.y *= 0.98;
        new_pos += self.vel.with_delta(delta);


        let mut top = new_pos;
        top.y-=8.0;
        let mut bottom = new_pos;
        bottom.y+=8.0;
        let mut left = new_pos;
        left.x-=8.0;
        let mut right = new_pos;
        right.x+=8.0;

        // let pix = map.heightmap.get_pixel(((self.pos.x*5.0)/10.0) as u32, ((self.pos.y*5.0)/10.0) as u32);
        // self.vel.x += (pix[1] as f32 - 125.0);
        // self.vel.y += (pix[2] as f32 - 125.0);

        if self.vel.y < 0.0 {
            if let Some(p) = map.get_point(&top.to_point()) {
                if matches!(p, GameObject::Wall {..}) {
                    self.vel.y*=-1.0;
                }
            }
        }else {
            if let Some(p) = map.get_point(&bottom.to_point()) {
                if matches!(p, GameObject::Wall {..}) {
                    self.vel.y*=-1.0;
                }
            }
        }

        if self.vel.x < 0.0 {
            if let Some(p) = map.get_point(&left.to_point()) {
                if matches!(p, GameObject::Wall {..}) {
                    self.vel.x*=-1.0;
                }
            }
        }else {
            if let Some(p) = map.get_point(&right.to_point()) {
                if matches!(p, GameObject::Wall {..}) {
                    self.vel.x*=-1.0;
                }
            }
        }

        if new_pos.x < 4.0 || new_pos.x > 400.0-6.0 {
            self.vel.x *= -1.0;
        }
        if new_pos.y < 4.0 || new_pos.y > 400.0-6.0 {
            self.vel.y *= -1.0;
        }

        self.pos += self.vel.with_delta(delta);
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
struct Pos {
    x: f32,
    y: f32,
}

impl Pos {
    // impl into
    fn new<T: Into<f32>>(x: T, y: T) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    fn with_delta(&self, delta: f32) -> Self {
        Self {
            x: self.x * delta,
            y: self.y * delta,
        }
    }

    fn velocity(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn to_point(&self) -> Point {
        Point { x: (self.x/20.0) as i32, y: (self.y/20.0) as i32 }
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Into<Vec2> for Pos {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Into<Pos2> for &mut Pos {
    fn into(self) -> Pos2 {
        Pos2::new(self.x, self.y)
    }
}
impl Into<Pos2> for Pos {
    fn into(self) -> Pos2 {
        Pos2::new(self.x, self.y)
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct GolfMap {
    objects: Vec<GameObject>,
    map: HashMap<Point, GameObject>,
    #[serde(skip)]
    heightmap: RgbImage,
}

fn char_i(index:i32)->String {
    let mut s = String::new();
    s.push(CHARS.chars().nth(index as usize).unwrap_or('~'));
    s
}

fn i_char(c:char)->i32 {
    CHARS.chars().position(|x| x == c).unwrap_or(0) as i32
}

impl GolfMap {


    fn to_text(&self) -> String {
        let mut s = String::new();

        for i in &self.objects {
            s=match i {
                GameObject::Wall { a:Point{x:-1,y:-1}, b:Point{x:-1,y:-1} } => {
                    "".to_owned()
                }
                GameObject::Wall { a, b } => {
                    format!("a{}{}{}{}", char_i(a.x), char_i(a.y), char_i(b.x), char_i(b.y))
                }
                GameObject::Hole(pos) => {
                    format!("b{}{}", char_i(pos.x), char_i(pos.y))
                }
                GameObject::Height { a, b, height } => {
                    format!("c{}{}{}{}{}", char_i(a.x), char_i(a.y), char_i(b.x), char_i(b.y), char_i(*height))
                }
                GameObject::Start(pos) => {
                    format!("d{}{}", char_i(pos.x), char_i(pos.y))
                }


            }+s.as_str();
        }
        s
    }

    fn from_text(&mut self, text: String) {
        self.map.clear();
        self.objects.clear();
        let mut i = 0;
        let chars: Vec<i32> = text.chars().map(|x| i_char(x)).collect();
        while i < chars.len() {
            let c = chars[i];

            match c  {
                0 => if chars.len() > i+5 {
                    
                    let a = Point { x: chars[i+1], y: chars[i+2] };
                    let b = Point { x: chars[i+3], y: chars[i+4] };
                    self.add_object(GameObject::Wall { a, b });
                    i+=5;
                }
                1 => if chars.len() > i+3 {
                    let pos = Point { x: chars[i+1], y: chars[i+2] };
                    self.add_object(GameObject::Hole(pos));
                    i+=3;
                }
                2 => if chars.len() > i+6 {
                    let a = Point { x: chars[i+1], y: chars[i+2] };
                    let b = Point { x: chars[i+3], y: chars[i+4] };
                    let height = chars[i+5];
                    self.add_object(GameObject::Height { a, b, height });
                    i+=6;
                }
                3 => if chars.len() > i+3 {
                    let pos = Point { x: chars[i+1], y: chars[i+2] };
                    self.add_object(GameObject::Start(pos));
                    i+=3;
                }
                _ => {}
            }
        }
    }

    fn add_object(&mut self, obj: GameObject) {
        self.objects.push(obj);
        self.update_hashmap();
    }
    fn get_point(&self, point: &Point) -> Option<&GameObject> {
        self.map.get(point)
    }

    fn update_heightmap(&mut self) {
        let mut image: RgbImage = RgbImage::from_pixel(20, 20, Rgb([125, 125, 125]));

        

        for i in &self.objects {
            match i {
                GameObject::Height { a, b, height } => {
                    for i in a.x.min(b.x)..=b.x.max(a.x) {
                        for j in a.y.min(b.y)..=b.y.max(a.y) {
                            let og = image.get_pixel(i as u32, j as u32).0[0];
                            
                            image.put_pixel(i as u32, j as u32, Rgb([*height as u8+og.clamp(0, 255) as u8, 0, 0]));
                        }
                    }
                }
                GameObject::Hole(pos) => {
                    image.put_pixel(pos.x as u32, pos.y as u32, Rgb([125-100, 125, 125]));
                }
                _ => {}
            }
        }

        
        
        image = DynamicImage::ImageRgb8(image).resize_exact(200, 200, FilterType::Triangle).to_rgb8();


        
        for x in 0..200{
            for y in 0..200 {
                let top = image.get_pixel(x, (y as i32-1).max(0) as u32);
                let bottom = image.get_pixel(x, (y+1).min(199));
                let left = image.get_pixel((x as i32-1).max(0) as u32, y);
                let right = image.get_pixel((x+1).min(199), y);
                let center = image.get_pixel(x, y);

                let green = (top[0] as i32-bottom[0] as i32 + 125).clamp(0, 256) as u8;
                let blue = (left[0] as i32-right[0] as i32 + 125).clamp(0, 256) as u8;

                image.put_pixel(x, y, Rgb([center[0], green, blue]));
                
            }
        }

        

    }

    fn update_hashmap(&mut self) {
        self.map.clear();
        for obj in &self.objects {
            match obj {
                GameObject::Wall { a, b } => {
                    for i in a.x.min(b.x)..=b.x.max(a.x) {
                        for j in a.y.min(b.y)..=b.y.max(a.y) {
                            self.map.insert(Point { x: i, y: j }, *obj);
                        }
                    }
                }
                _ => {
                    if let Some(p) = obj.point() {
                        self.map.insert(p, *obj);
                    }
                }
            }
        }
    }
}

impl Default for GameObject {
    fn default() -> Self {
        Self::Start(Point::default())
    }
}

impl Default for GolfMap {
    fn default() -> Self {
        
        Self {
            objects: vec![GameObject::Start(Point::default())],
            map: HashMap::new(),
            heightmap: RgbImage::from_pixel(200, 200, Rgb([125, 125, 125])),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    map: GolfMap,
    ball: GolfBall,
    edit: EditOption,
    reset: bool,
    slider: i32,
    scale: f32,
    text: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            map: GolfMap::default(),
            ball: GolfBall::default(),
            edit: EditOption::PlayGame,
            reset: true,
            slider: 0,
            scale: 3.0,
            text: String::new(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        // set to continuous updates
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.


        println!("new");
        let mut new: Self = Default::default();
        #[cfg(target_arch = "wasm32")]
        match web_sys::window() {
            Some(window) => {
                
                println!("window: {:?}",window.location().as_string());
            }
            None => {
                println!("no window");
            }
        }

        if let Some(storage) = cc.storage {
            new = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        new.reset();
        new
    }

    fn reset(&mut self) {
        let mut has_start = false;
        for i in &self.map.objects {
            match i {
                GameObject::Start(p) => {
                    has_start = true;
                    self.ball.pos = Pos::new(p.x as f32 * 20.0 + 10.0, p.y as f32 * 20.0 + 10.0);
                }
                _ => {}
            }
        }
        if !has_start {
            self.map.objects.push(GameObject::Start(Point::default()));
            self.ball.pos = Pos::new(10.0, 10.0);
        }

        match self.edit {
            EditOption::EditMap(GameObject::Wall { a: _, b: _ }) => {
                self.map.objects.push(GameObject::Wall {
                    a: Point { x: -1, y: -1 },
                    b: Point { x: -1, y: -1 },
                });
            }
            _ => {}
        }

        self.map.update_hashmap();
        self.ball.vel = Pos::default();

        self.map.update_heightmap();
        self.text = self.map.to_text();
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {


        egui::TopBottomPanel::bottom("bottom panel").show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.text);
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if ui.button("Reset").clicked() {
                self.reset = true;
            };
            // ui elements to edit the game
            ui.separator();
            if ui
                .selectable_label(self.edit == EditOption::PlayGame, "Play Game")
                .clicked()
            {
                self.edit = EditOption::PlayGame;
            };
            ui.separator();
            if ui.button("clear map").clicked() {
                self.map.objects.clear();
                self.map.objects.push(GameObject::Start(Point {x:1, y:1}));
                self.map.objects.push(GameObject::Hole(Point {x:18, y:18}));
                self.map.update_hashmap();
                self.reset();
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.small_button("-").clicked() {
                    self.scale *= 0.75;
                }
                ui.add(egui::DragValue::new(&mut self.scale).speed(0.001).clamp_range(0.0..=4.0).prefix("Scale: "));
                if ui.small_button("+").clicked() {
                    self.scale *= 1.25;
                }
            });
            if ui
                .selectable_label(self.edit == EditOption::Delete, "Delete")
                .clicked()
            {
                self.edit = EditOption::Delete;
            };

            if ui
                .selectable_label(
                    self.edit == EditOption::EditMap(GameObject::Start(Point::default())),
                    "Move Start",
                )
                .clicked()
            {
                self.edit = EditOption::EditMap(GameObject::Start(Point::default()));
            };

            if ui
                .selectable_label(
                    self.edit == EditOption::EditMap(GameObject::Hole(Point::default())),
                    "Move Hole",
                )
                .clicked()
            {
                self.edit = EditOption::EditMap(GameObject::Hole(Point::default()));
            };



            if ui
                .selectable_label(
                    matches!(self.edit
                        , EditOption::EditMap(GameObject::Wall {
                            a: _,
                            b: _,
                        })),
                    "Add Wall",
                )
                .clicked()
            {
                self.edit = EditOption::EditMap(GameObject::Wall {
                    a: Point { x: -1, y: -1 },
                    b: Point { x: -1, y: -1 },
                });
            };
            ui.separator();
            ui.small("folowing options are not implemented yet and will be avalible in a future version");
            if ui
                .selectable_label(
                    matches!(self.edit
                        , EditOption::EditMap(GameObject::Height {
                            a: _,
                            b: _,
                            height: _,
                        })),
                    "Add Height",
                )
                .clicked()
            {
                self.edit = EditOption::EditMap(GameObject::Height {
                    a: Point { x: -1, y: -1 },
                    b: Point { x: -1, y: -1 },
                    height: 0,
                });
            };

            ui.add(egui::Slider::new(&mut self.slider, -40..=40).integer().drag_value_speed(0.1).prefix("Height: "));

            
            

        });

        if self.reset {
            self.reset();
            self.reset = false;
        }

        let Self {
            map,
            edit,
            ball,
            scale,
            ..
        } = self;
        
        // if not web then print hello
        


        #[cfg(not(target_arch = "wasm32"))]
        ctx.set_pixels_per_point(*scale);

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let mut delta = 0.0;
        ctx.input(|i| {
            delta = i.predicted_dt;
        });

        // update movement of ball every frame
        ball.update_pos(map, delta);

        let mut clicked_point = None;



        egui::CentralPanel::default().show(ctx, |ui| {
            let mut clicked = false;
            ctx.input(|o| {
                clicked = o.events.iter().any(|e| {
                    matches!(
                        e,
                        egui::Event::PointerButton {
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            ..
                        }
                    )
                });
            });

            // spawn playing green
            let green_rect = egui::Area::new("green_area")
                .movable(true)
                .show(ctx, |ui| {
                    egui::Frame::default()
                        .rounding(1.0)
                        .fill(egui::Color32::from_rgb(0, 136, 84))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 0.0);

                            // spawn buttons
                            for y in 0..20 {
                                ui.horizontal(|ui| {
                                    for x in 0..20 {
                                        
                                        let text = match map.get_point(&Point { x, y }) {
                                            _ if matches!(edit, EditOption::PlayGame) => " ".to_owned(),
                                            Some(a) => a.symbol(),

                                            _ => " ".to_owned(),
                                        };
                                        if ui
                                            .add_sized(
                                                [20.0, 20.0],
                                                egui::Button::new(text)
                                                    .small()
                                                    .rounding(match map.get_point(&Point { x, y }) {
                                                        Some(GameObject::Hole(_)) => 20.0,
                                                        _ => 0.0
                                                    })
                                                    .stroke(match map.get_point(&Point { x, y }) {
                                                        Some(GameObject::Hole(_)) => Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0)),
                                                        _ => Stroke::new(0.0, egui::Color32::from_rgb(0, 0, 0))
                                                    })
                                                    .min_size(egui::Vec2::new(20.0, 20.0))
                                                    .fill(match 
                                                        map.get_point(&Point { x, y}) {
                                                            Some(GameObject::Wall { a:_, b:_ }) => egui::Color32::from_rgb(0, 0, 0),
                                                            _ => egui::Color32::from_rgb(0, 136, 84)
                                                        }
                                                    ),
                                            )
                                            .clicked()
                                        {
                                            clicked_point = Some(Point { x, y });
                                        }
                                    }
                                });
                            }
                        });
                    ui.label("move me");
                })
                .response
                .rect;

            

            let ball_pos = green_rect.left_top() + ball.pos.into();
            let painter = ui
                .painter()
                .with_clip_rect(green_rect)
                .with_layer_id(egui::LayerId {
                    order: egui::Order::Tooltip,
                    id: egui::Id::new("ball"),
                });
            painter.circle(
                ball_pos,
                8.0,
                egui::Color32::WHITE,
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            );

            if let Some(pointer) = ctx.pointer_latest_pos() {
                if ball_pos.distance(pointer) < 80.0
                    && ball.vel.velocity() < 1.0
                    && matches!(edit, EditOption::PlayGame)
                {
                    ui.output_mut(|o| {
                        o.cursor_icon = egui::CursorIcon::Crosshair;
                    });
                    let mut clone = ball.clone();
                    clone.vel = Pos::new(
                        -(pointer.x - ball_pos.x) * 12.0,
                        -(pointer.y - ball_pos.y) * 12.0,
                    );
                    if clicked {
                        *ball = clone;
                    }
                    for i in (0..14).rev() {
                        for _ in 0..3 {
                            clone.update_pos(&map, delta)
                        }
                        painter.circle_filled(
                            green_rect.left_top() + clone.pos.into(),
                            (i as f32 / 4.0) + 4.0,
                            egui::Color32::from_rgba_premultiplied(180, 180, 180, 180),
                        );
                    }
                }
            }
            match edit {
                EditOption::EditMap(GameObject::Wall {a:c, b:_}) if !matches!(c, Point {x:-1,y:-1}) => {
                    let x1 = c.x as f32*20.0 + green_rect.left()+10.0;
                    let y1 = c.y as f32*20.0 + green_rect.top()+10.0;
                    let x2 = ((ctx.pointer_hover_pos().unwrap_or_default().x - green_rect.left())/20.0).floor()*20.0+10.0 + green_rect.left();
                    let y2 = ((ctx.pointer_hover_pos().unwrap_or_default().y - green_rect.top())/20.0).floor()*20.0+10.0 + green_rect.top();
                    painter.rect(Rect { min: Pos2 { x:x1.min(x2), y:y1.min(y2) }, max:  Pos2 { x:x1.max(x2), y:y1.max(y2) } }, 3.0, Color32::from_black_alpha(100), Stroke::new(10.0, Color32::from_black_alpha(100)));
                }
                _ => {}
            }
        });
        if ball.vel.velocity() > 1.0 {
            ctx.request_repaint_after(Duration::from_millis(16));
        }


        // if a point was clicked, the it is handled here, outside the egui context
        if let Some(pt) = clicked_point {
            match &edit {
                EditOption::EditMap(GameObject::Start(_)) => {
                    map.objects.retain(|i| !matches!(i, GameObject::Start(_)));
                    map.objects.push(GameObject::Start(pt));
                    self.reset();
                }
                EditOption::EditMap(GameObject::Hole(_)) => {
                    map.objects.retain(|i| !matches!(i, GameObject::Hole(_)));
                    map.objects.push(GameObject::Hole(pt));
                    self.reset();
                }

                EditOption::EditMap(GameObject::Height {
                    a: Point { x: -1, y: -1 },
                    b: _,
                    height: h,
                }) => {
                    self.edit = EditOption::EditMap(GameObject::Height {
                        a: pt,
                        b: Point { x: -1, y: -1 },
                        height: *h,
                    });
                }

                EditOption::EditMap(GameObject::Height { a, b: _ , height:h}) => {
                    self.map.add_object(GameObject::Height { a: *a, b: pt, height: self.slider });
                    self.edit = EditOption::EditMap(GameObject::Height {
                        a: Point { x: -1, y: -1 },
                        b: Point { x: -1, y: -1 },
                        height: self.slider,
                    });
                    self.reset();
                }

                EditOption::EditMap(GameObject::Wall {
                    a: Point { x: -1, y: -1 },
                    b: _,
                }) => {
                    self.edit = EditOption::EditMap(GameObject::Wall {
                        a: pt,
                        b: Point { x: -1, y: -1 },
                    });
                }

                EditOption::EditMap(GameObject::Wall { a, b: _ }) => {
                    self.map.add_object(GameObject::Wall { a: *a, b: pt });
                    self.edit = EditOption::EditMap(GameObject::Wall {
                        a: Point { x: -1, y: -1 },
                        b: Point { x: -1, y: -1 },
                    });
                    self.reset();
                }
                EditOption::Delete => {
                    self.map.objects.retain(|f| !f.get_points().contains(&pt));
                    self.reset();
                }
                

                _ => {}
            }
        }
    }
}
