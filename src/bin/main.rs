use pgrep::App;

fn main() -> pgrep::Result<()> {
  App::new()?.run()
}
