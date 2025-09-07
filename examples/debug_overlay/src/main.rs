use astraweave_core::{
    ActionStep, CompanionState, EnemyState, IVec2, PlanIntent, PlayerState, Team, World,
    WorldSnapshot,
};
use eframe::egui;

struct App {
    grid_w: i32,
    grid_h: i32,
    world: World,
    player: u32,
    comp: u32,
    enemy: u32,
    plan: Option<PlanIntent>,
}

impl App {
    fn new() -> Self {
        let mut world = World::new();
        // wall
        for y in 1..=8 {
            world.obstacles.insert((6, y));
        }
        let player = world.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
        let comp = world.spawn("Comp", IVec2 { x: 2, y: 3 }, Team { id: 1 }, 80, 30);
        let enemy = world.spawn("Enemy", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);
        // trivial plan just to show rendering
        let plan = Some(PlanIntent {
            plan_id: "viz".into(),
            steps: vec![
                ActionStep::MoveTo { x: 4, y: 2 },
                ActionStep::Throw {
                    item: "smoke".into(),
                    x: 7,
                    y: 2,
                },
                ActionStep::CoverFire {
                    target_id: enemy,
                    duration: 2.0,
                },
            ],
        });
        Self {
            grid_w: 20,
            grid_h: 10,
            world,
            player,
            comp,
            enemy,
            plan,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("AstraWeave Debug Overlay");
            ui.label("Blue=Player, Green=Companion, Red=Enemy, Black=Obstacles");
            let desired_size = egui::vec2(600.0, 320.0);
            let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
            let mut shapes = vec![];
            let cell_w = rect.width() / self.grid_w as f32;
            let cell_h = rect.height() / self.grid_h as f32;

            let to_screen = |p: IVec2| -> egui::Pos2 {
                egui::pos2(
                    rect.left() + p.x as f32 * cell_w,
                    rect.top() + p.y as f32 * cell_h,
                )
            };

            // draw grid
            for x in 0..=self.grid_w {
                let xpx = rect.left() + x as f32 * cell_w;
                shapes.push(egui::Shape::line_segment(
                    [egui::pos2(xpx, rect.top()), egui::pos2(xpx, rect.bottom())],
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                ));
            }
            for y in 0..=self.grid_h {
                let ypx = rect.top() + y as f32 * cell_h;
                shapes.push(egui::Shape::line_segment(
                    [egui::pos2(rect.left(), ypx), egui::pos2(rect.right(), ypx)],
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                ));
            }

            // obstacles
            for (x, y) in self.world.obstacles.iter() {
                let r = egui::Rect::from_min_size(
                    egui::pos2(
                        rect.left() + *x as f32 * cell_w,
                        rect.top() + *y as f32 * cell_h,
                    ),
                    egui::vec2(cell_w, cell_h),
                );
                shapes.push(egui::Shape::rect_filled(r, 0.0, egui::Color32::BLACK));
            }

            // entities
            let p = self.world.pos_of(self.player).unwrap();
            let c = self.world.pos_of(self.comp).unwrap();
            let e = self.world.pos_of(self.enemy).unwrap();
            let draw_dot = |p: IVec2, col: egui::Color32| -> egui::Shape {
                let center = to_screen(p);
                egui::Shape::circle_filled(center, (cell_w.min(cell_h) * 0.35) as f32, col)
            };
            shapes.push(draw_dot(p, egui::Color32::from_rgb(80, 160, 255)));
            shapes.push(draw_dot(c, egui::Color32::from_rgb(80, 240, 120)));
            shapes.push(draw_dot(e, egui::Color32::from_rgb(240, 80, 80)));

            // plan arrows
            if let Some(plan) = &self.plan {
                let mut cur = c;
                for step in &plan.steps {
                    match step {
                        ActionStep::MoveTo { x, y } => {
                            let nxt = IVec2 { x: *x, y: *y };
                            shapes.push(egui::Shape::line_segment(
                            [to_screen(cur), to_screen(nxt)],
                             egui::Stroke::new(2.0, egui::Color32::YELLOW),
                        ));

                            cur = nxt;
                        }
                        ActionStep::Throw { item: _, x, y } => {
                            let tgt = IVec2 { x: *x, y: *y };
                            shapes.push(egui::Shape::circle_stroke(
                                to_screen(tgt),
                                6.0,
                                egui::Stroke::new(2.0, egui::Color32::GOLD),
                            ));
                        }
                        ActionStep::CoverFire {
                            target_id: _,
                            duration: _,
                        } => {
                            shapes.push(egui::Shape::text(
                                &egui::FontId::proportional(12.0),
                                to_screen(cur),
                                "cover fire",
                                egui::Align2::LEFT_TOP,
                                egui::Color32::WHITE,
                            ));
                        }
                        _ => {}
                    }
                }
            }

            ui.painter().extend(shapes);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native = eframe::NativeOptions::default();
    eframe::run_native(
        "AstraWeave Debug Overlay",
        native,
        Box::new(|_| ok(Box::new(App::new()))),
    )
}
