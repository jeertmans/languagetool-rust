use codspeed_criterion_compat::criterion_main;

mod benchmarks;
criterion_main! {
    benchmarks::check_texts::checks,

}
