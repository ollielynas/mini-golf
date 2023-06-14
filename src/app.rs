use egui::{Pos2, Rect, Vec2};
use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
    time::Duration,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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

        // iter over all tiles in a 1 tile raduis

        let mut tiles = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                tiles.push(Point {
                    x: new_pos.x as i32 + x,
                    y: new_pos.y as i32 + y,
                });
            }
        }

        for t in tiles {
            if let Some(res) = map.get_point(&t) {
                println!("hit something");
                match res {
                    GameObject::Wall { .. } => {
                        println!("hit wall");
                        let mut new_vel = self.vel.clone();
                        if t.x == new_pos.x as i32 {
                            new_vel.x *= -1.0;
                        }
                        if t.y == new_pos.y as i32 {
                            new_vel.y *= -1.0;
                        }
                        self.vel = new_vel;
                    }
                    _ => {}
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
    heightmap: HashMap<Point, i32>,
}

impl GolfMap {
    fn add_object(&mut self, obj: GameObject) {
        self.objects.push(obj);
        self.update_hashmap();
    }
    fn get_point(&self, point: &Point) -> Option<&GameObject> {
        self.map.get(point)
    }

    fn update_heightmap(&mut self) {
        self.heightmap.clear();
        for i in &self.objects {
            match i {
                GameObject::Height { a, b, height } => {
                    let mut x = a.x;
                    let mut y = a.y;
                    while x != b.x || y != b.y {
                        self.heightmap.insert(Point { x, y }, *height);
                        if x < b.x {
                            x += 1;
                        } else if x > b.x {
                            x -= 1;
                        }
                        if y < b.y {
                            y += 1;
                        } else if y > b.y {
                            y -= 1;
                        }
                    }
                }
                _ => {}
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
            heightmap: HashMap::new(),
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
}

impl Default for App {
    fn default() -> Self {
        Self {
            map: GolfMap::default(),
            ball: GolfBall::default(),
            edit: EditOption::PlayGame,
            reset: true,
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
        let mut new: Self = Default::default();
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
                    b: Point::default(),
                });
            }
            _ => {}
        }

        self.map.update_hashmap();
        self.ball.vel = Pos::default();

        self.map.update_heightmap();
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
                self.map.objects.push(GameObject::Start(Point::default()));
                self.map.update_hashmap();
            }
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
                    self.edit
                        == EditOption::EditMap(GameObject::Wall {
                            a: Point::default(),
                            b: Point::default(),
                        }),
                    "Add Wall",
                )
                .clicked()
            {
                self.edit = EditOption::EditMap(GameObject::Wall {
                    a: Point { x: -1, y: -1 },
                    b: Point::default(),
                });
            };

            if ui
                .selectable_label(
                    self.edit
                        == EditOption::EditMap(GameObject::Height {
                            a: Point::default(),
                            b: Point::default(),
                            height: 0,
                        }),
                    "Add Height",
                )
                .clicked()
            {
                self.edit = EditOption::EditMap(GameObject::Height {
                    a: Point { x: -1, y: -1 },
                    b: Point::default(),
                    height: 0,
                });
            };
            

        });

        if self.reset {
            self.reset();
            self.reset = false;
        }

        let Self {
            map,
            edit,
            ball,
            reset,
        } = self;

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
                                                    .min_size(egui::Vec2::new(20.0, 20.0))
                                                    .frame(false)
                                                    .fill(match 
                                                        map.get_point(&Point { x, y}) {
                                                            Some(GameObject::Wall { a:_, b:_ }) => egui::Color32::from_rgb(0, 0, 0),
                                                            _ => match map.heightmap.get(&Point { x, y }) {
                                                                Some(h) => egui::Color32::from_white_alpha((180+*h) as u8),
                                                                _ => egui::Color32::from_rgb(0, 136, 84),
                                                            }
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
                        -(pointer.x - ball_pos.x) * 18.0,
                        -(pointer.y - ball_pos.y) * 18.0,
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

                

                _ => {}
            }
        }
    }
}
