#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MiningMode { OneByOne, ThreeByOne, ThreeByTwo, ThreeByThree }
impl Default for MiningMode { fn default() -> Self { Self::ThreeByOne } }
impl MiningMode { fn footprint(self) -> (i32, i32) { match self { Self::OneByOne => (1,1), Self::ThreeByOne => (3,1), Self::ThreeByTwo => (3,2), Self::ThreeByThree => (3,3) } } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MovementMode { Walk, Mixed, Sprint, SprintJump }
impl Default for MovementMode { fn default() -> Self { Self::Mixed } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pos { x:i32, y:i32, z:i32 }
#[derive(Clone, Debug)]
struct State { open:bool, enabled:bool, start:Option<Pos>, end:Option<Pos>, mode:MiningMode, movement:MovementMode, route:Vec<Pos>, cursor:usize }
impl Default for State { fn default()->Self { Self{open:false,enabled:false,start:None,end:None,mode:MiningMode::default(),movement:MovementMode::default(),route:Vec::new(),cursor:0} } }
static STATE:std::sync::Mutex<Option<State>>=std::sync::Mutex::new(None);
fn state()->std::sync::MutexGuard<'static,Option<State>>{ STATE.lock().unwrap_or_else(std::sync::PoisonError::into_inner) }
impl State {
 fn rebuild(&mut self){
  self.route.clear(); self.cursor=0;
  let (Some(a),Some(b))=(self.start,self.end) else{return;};
  let min=Pos{x:a.x.min(b.x),y:a.y.min(b.y),z:a.z.min(b.z)};
  let max=Pos{x:a.x.max(b.x),y:a.y.max(b.y),z:a.z.max(b.z)};
  let (vy,hx)=self.mode.footprint();
  let mut y=min.y;
  while y<=max.y {
   let mut z=min.z;
   while z<=max.z {
    let mut x=min.x;
    while x<=max.x {
     for dy in 0..vy { for dx in 0..hx { let px=x+dx; let py=y+dy; if px<=max.x && py<=max.y { self.route.push(Pos{x:px,y:py,z}); } } }
     x+=hx;
    }
    z+=1;
   }
   y+=vy;
  }
 }
}
impl crate::game::Game<'_> {
 pub(super) fn functionrs_insert(&mut self,key:winit::keyboard::KeyCode,pressed:bool,repeat:bool){
  if !pressed || repeat{return;}
  match key {
   winit::keyboard::KeyCode::Insert=>{let mut g=state();let s=g.as_mut().unwrap();s.open=!s.open;self.system_message(horizon_client::text::Text::literal(if s.open{"Auto Miner OPEN: C=Start E=End 1-4=Mode M=Movement R=Run X=Stop"}else{"Auto Miner CLOSED"}));}
   winit::keyboard::KeyCode::KeyC=>self.functionrs_set_start(),
   winit::keyboard::KeyCode::KeyE=>self.functionrs_set_end(),
   winit::keyboard::KeyCode::Digit1=>self.functionrs_set_mode(MiningMode::OneByOne),
   winit::keyboard::KeyCode::Digit2=>self.functionrs_set_mode(MiningMode::ThreeByOne),
   winit::keyboard::KeyCode::Digit3=>self.functionrs_set_mode(MiningMode::ThreeByTwo),
   winit::keyboard::KeyCode::Digit4=>self.functionrs_set_mode(MiningMode::ThreeByThree),
   winit::keyboard::KeyCode::KeyM=>self.functionrs_cycle_movement(),
   winit::keyboard::KeyCode::KeyR=>self.functionrs_start(),
   winit::keyboard::KeyCode::KeyX=>self.functionrs_stop(),
   _=>{}
  }
 }
 fn functionrs_set_start(&mut self){let Some(p)=self.player.as_ref()else{return;};let pos=Pos{x:p.position.x.floor()as i32,y:(p.position.y-0.001).floor()as i32,z:p.position.z.floor()as i32};let mut g=state();let s=g.as_mut().unwrap();s.start=Some(pos);s.rebuild();self.system_message(horizon_client::text::Text::literal("Auto Miner: START saved"));}
 fn functionrs_set_end(&mut self){let Some(p)=self.player.as_ref()else{return;};let pos=Pos{x:p.position.x.floor()as i32,y:(p.position.y-0.001).floor()as i32,z:p.position.z.floor()as i32};let mut g=state();let s=g.as_mut().unwrap();s.end=Some(pos);s.rebuild();self.system_message(horizon_client::text::Text::literal("Auto Miner: END saved"));}
 fn functionrs_set_mode(&mut self,m:MiningMode){let mut g=state();let s=g.as_mut().unwrap();s.mode=m;s.rebuild();self.system_message(horizon_client::text::Text::literal(match m{MiningMode::OneByOne=>"Auto Miner: Mode 1 (1x1)",MiningMode::ThreeByOne=>"Auto Miner: Mode 2 (3x1)",MiningMode::ThreeByTwo=>"Auto Miner: Mode 3 (3x2)",MiningMode::ThreeByThree=>"Auto Miner: Mode 4 (3x3)"}));}
 fn functionrs_cycle_movement(&mut self){let mut g=state();let s=g.as_mut().unwrap();s.movement=match s.movement{MovementMode::Walk=>MovementMode::Mixed,MovementMode::Mixed=>MovementMode::Sprint,MovementMode::Sprint=>MovementMode::SprintJump,MovementMode::SprintJump=>MovementMode::Walk};self.system_message(horizon_client::text::Text::literal(match s.movement{MovementMode::Walk=>"Auto Miner: Walk",MovementMode::Mixed=>"Auto Miner: Mixed",MovementMode::Sprint=>"Auto Miner: Sprint",MovementMode::SprintJump=>"Auto Miner: Sprint + Jump"}));}
 fn functionrs_start(&mut self){let mut g=state();let s=g.as_mut().unwrap();if s.start.is_some()&&s.end.is_some()&&!s.route.is_empty(){s.cursor=0;s.enabled=true;self.system_message(horizon_client::text::Text::literal("Auto Miner: RUNNING"));}else{self.system_message(horizon_client::text::Text::literal("Auto Miner: set START and END first"));}}
 fn functionrs_stop(&mut self){let mut g=state();g.as_mut().unwrap().enabled=false;self.system_message(horizon_client::text::Text::literal("Auto Miner: STOPPED"));}
 pub(super) fn functionrs_level_tick(&mut self){
  if !self.has_singleplayer_server(){return;}
  let target={let g=state();let s=g.as_ref().unwrap();if !s.enabled{return;}match s.route.get(s.cursor).copied(){Some(p)=>p,None=>{drop(g);let mut g2=state();g2.as_mut().unwrap().enabled=false;return;}}};
  let Some(player)=self.player.as_ref()else{return;};
  let distance=((player.position.x-(f64::from(target.x)+0.5)).powi(2)+(player.position.y-f64::from(target.y)).powi(2)+(player.position.z-(f64::from(target.z)+0.5)).powi(2)).sqrt();
  if distance>4.5{drop(player);self.functionrs_move(target);return;}
  if self.world.loaded(target.x,target.z){let before=self.world.block(target.x,target.y,target.z);if !self.world.is_air(before){let _=self.break_block_as_player([target.x,target.y,target.z]);}}
  let mut g=state();let s=g.as_mut().unwrap();s.cursor+=1;if s.cursor>=s.route.len(){s.enabled=false;self.system_message(horizon_client::text::Text::literal("Auto Miner: FINISHED"));}
 }
 fn functionrs_move(&mut self,target:Pos){let movement={let g=state();g.as_ref().unwrap().movement};let Some(p)=self.player.as_mut()else{return;};let dx=f64::from(target.x)+0.5-p.position.x;let dz=f64::from(target.z)+0.5-p.position.z;if dx.abs()+dz.abs()<0.01{return;}p.yaw=(-(dx.atan2(dz)).to_degrees())as f32;p.head_yaw=p.yaw;let jump=matches!(movement,MovementMode::SprintJump);let _=self.motion.player_movement_tick([0.0,1.0],jump);if matches!(movement,MovementMode::Sprint|MovementMode::SprintJump){let _=self.motion.sprint_input_tick(horizon_client::sprint::Keys{forward:true,sprint:true,jump,..Default::default()});}}
}