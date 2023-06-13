use std::{ops::{Add, AddAssign}, time::Duration, collections::HashMap};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use egui::{Vec2, Pos2, Rect};

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
enum EditOption {
    PlayGame,
    EditMap(GameObject),
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
enum GameObject {
    Hole(Point),
    Wall{a: Point, b: Point},
    Start(Point),
    DipS(Point),
    DipM(Point),
    DipL(Point),
    HillS(Point),
    HillM(Point),
    HillL(Point),
}

impl GameObject {
    fn symbol(&self) -> String {
        match &self {
            GameObject::Hole(_) => "H".to_string(),
            GameObject::Wall{..} => "W".to_string(),
            GameObject::Start(_) => "S".to_string(),
            GameObject::DipS(_) => "DS".to_string(),
            GameObject::DipM(_) => "DM".to_string(),
            GameObject::DipL(_) => "DL".to_string(),
            GameObject::HillS(_) => "HS".to_string(),
            GameObject::HillM(_) => "HM".to_string(),
            GameObject::HillL(_) => "HL".to_string(),
        }
    }

    fn point(&self) -> Option<Point> {
        match &self {
            GameObject::Hole(p) => Some(*p),
            GameObject::Wall{..} => None,
            GameObject::Start(p) => Some(*p),
            GameObject::DipS(p) => Some(*p),
            GameObject::DipM(p) => Some(*p),
            GameObject::DipL(p) => Some(*p),
            GameObject::HillS(p) => Some(*p),
            GameObject::HillM(p) => Some(*p),
            GameObject::HillL(p) => Some(*p),
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
        Self {
            x: 0,
            y: 0,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
struct GolfBall {
    pos: Pos,
    vel: Pos,
    shoot: bool
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
            x: self.x+rhs.x,
            y: self.y+rhs.y,
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
        self.vel.x*=0.98;
        self.vel.y*=0.98;
        new_pos += self.vel.with_delta(delta);
        
        if new_pos.x < 4.0 || new_pos.x > 400.0 {
            self.vel.x *= -1.0;
        }
        if new_pos.y < 4.0 || new_pos.y > 400.0 {
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
            x: self.x*delta,
            y: self.y*delta,
        }
    }

    fn velocity(&self) -> f32 {
        (self.x.powi(2)+self.y.powi(2)).sqrt()
    }

    
}

impl Default for Pos {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
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
}

impl GolfMap {
    fn add_object(&mut self, obj: GameObject) {
        self.objects.push(obj);
        self.update_hashmap();
    }
    fn get_point(&self, point:&Point) -> Option<&GameObject> {
        self.map.get(point)
    }
    fn update_hashmap(&mut self) {
        self.map.clear();
        for obj in &self.objects {
            match obj {
                GameObject::Wall{a, b} => {
                    let mut x = a.x;
                    let mut y = a.y;
                    while x != b.x || y != b.y {
                        self.map.insert(Point{x, y}, *obj);
                        if x < b.x {
                            x+=1;
                        } else if x > b.x {
                            x-=1;
                        }
                        if y < b.y {
                            y+=1;
                        } else if y > b.y {
                            y-=1;
                        }
                    }
                },
                _ => {
                    if let Some(p) = obj.point() {
                        self.map.insert(p, *obj);
                    }
                }
            }
            
        }
    }
}

impl Default for GolfMap {
    fn default() -> Self {
        Self {
            objects: vec![GameObject::Start(Point::default())],
            map: HashMap::new(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    map: GolfMap,
    ball: GolfBall,
    edit: EditOption,
}

impl Default for App {
    fn default() -> Self {
        Self {
            map: GolfMap::default(),
            ball: GolfBall::default(),
            edit: EditOption::PlayGame,
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
        let mut new:Self =  Default::default();
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
                    self.ball.pos = Pos::new(p.x as f32*20.0+10.0, p.y as f32*20.0+10.0);
                }
                _ => {}
            }
        }
        if !has_start {
            self.map.objects.push(GameObject::Start(Point::default()));
            self.ball.pos = Pos::new(10.0, 10.0);
        }
        self.map.update_hashmap();
        self.ball.vel = Pos::default();
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
        
        let mut reset = false;
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");
            if ui.button( "Reset").clicked() {
                reset = true;
            };
            ui.separator();
            if ui.selectable_label(self.edit == EditOption::PlayGame, "Play Game").clicked() {
                self.edit = EditOption::PlayGame;
            };

            for i in GameObject::iter() {
                if ui.selectable_label(self.edit == EditOption::Object(i), i.to_string()).clicked() {
                    self.edit = EditOption::Object(i);
                };
            }

        });
        
        if reset {
            self.reset();
        }

        let Self {map ,edit, ball} = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let mut delta = 0.0;
        ctx.input(|i| {
            delta = i.predicted_dt;
        });

        

        ball.update_pos(map, delta);

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut clicked = false;
            ctx.input(|o| {
                clicked = o.events.iter().any(|e| matches!(e, egui::Event::PointerButton { button: egui::PointerButton::Primary, pressed: true, .. }));
            });
            let green_rect = egui::Area::new("green_area")
            .movable(true).show(ctx, |ui| {
                egui::Frame::default()
                .rounding(1.0)
                .fill(egui::Color32::from_rgb(0, 136, 84)).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(0.0,0.0);
                    for y in 0..20 {
                        ui.horizontal(|ui| {
                            for x in 0..20 {
                                let text = match map.get_point(&Point{x, y}) {
                                    Some(a) => a.symbol(),
                                    
                                    _ => " ".to_owned(),
                                };
                                ui.add_sized([20.0,20.0],egui::Button::new(text)
                                .small().min_size(egui::Vec2::new(20.0,20.0)).frame(false).fill(egui::Color32::from_rgb(0, 136, 84))
                            );
                            }
                        });
                    }
                });
            }).response.rect;


            let ball_pos = green_rect.left_top() + ball.pos.into();
            let painter = ui.painter().with_clip_rect(green_rect).with_layer_id(egui::LayerId { order: egui::Order::Tooltip, id: egui::Id::new("ball") });
            painter.circle(ball_pos, 8.0, egui::Color32::WHITE,egui::Stroke::new(1.0, egui::Color32::BLACK));

            if let Some(pointer) = ctx.pointer_latest_pos() {
                if ball_pos.distance(pointer) < 40.0 && ball.vel.velocity() < 1.0 {
                    ui.output_mut(|o| {
                        o.cursor_icon = egui::CursorIcon::Crosshair;
                    });
                    let mut clone = ball.clone();
                    clone.vel = Pos::new(-(pointer.x - ball_pos.x)*18.0, -(pointer.y - ball_pos.y)*18.0);
                    if clicked {
                        *ball = clone;
                    }
                    for i in (0..14).rev() {
                        for _ in 0..3 {clone.update_pos(&map, delta)};
                        painter.circle_filled(green_rect.left_top()+clone.pos.into(), (i as f32/4.0)+4.0, egui::Color32::from_rgba_premultiplied(180, 180, 180, 180));
                    }
                }
            }
        });
        if ball.vel.velocity() > 1.0 {
            ctx.request_repaint_after(Duration::from_millis(16));
        }



    }
}
