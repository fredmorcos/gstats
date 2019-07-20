use log::info;
use rand::Rng;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "bpdaggen", about = "Generate random bipartite DAGs")]
struct Opt {
    #[structopt(name = "n_vertices", help = "Number of vertices")]
    vertices: usize,
}

fn main() {
    env_logger::init();

    let opts = Opt::from_args();

    let mut rng = rand::thread_rng();

    let mut nodes: Vec<(usize, usize, usize)> = Vec::with_capacity(opts.vertices);
    let mut reds: Vec<usize> = Vec::with_capacity(opts.vertices / 2);
    let mut blues: Vec<usize> = Vec::with_capacity(opts.vertices / 2);

    // The root vertex.
    nodes.push((0, 0, 0));
    reds.push(1);

    // The second vertex.
    nodes.push((1, 1, rng.gen_range(nodes[0].2, 100)));
    blues.push(2);

    for i in 3..=opts.vertices + 1 {
        let bag: bool = rng.gen();

        if bag {
            let left = rng.gen_range(0, blues.len());
            let right = rng.gen_range(0, blues.len());

            info!("RED left = {}, right = {}", left, right);

            let left = blues[left];
            let right = blues[right];

            info!(">>>> left = {}, right = {}", left, right);

            let min_timestamp = nodes[left - 1].2.max(nodes[right - 1].2);
            let min_timestamp = min_timestamp + rng.gen_range(1, 100);
            let max_timestamp = min_timestamp + rng.gen_range(1, 100);
            let timestamp = rng.gen_range(min_timestamp, max_timestamp + 1);

            nodes.push((left, right, timestamp));
            reds.push(i);
        } else {
            let left = rng.gen_range(0, reds.len());
            let right = rng.gen_range(0, reds.len());

            info!("BLUE left = {}, right = {}", left, right);

            let left = reds[left];
            let right = reds[right];

            info!(">>>> left = {}, right = {}", left, right);

            let min_timestamp = nodes[left - 1].2.max(nodes[right - 1].2);
            let min_timestamp = min_timestamp + rng.gen_range(1, 100);
            let max_timestamp = min_timestamp + rng.gen_range(1, 100);
            let timestamp = rng.gen_range(min_timestamp, max_timestamp + 1);

            nodes.push((left, right, timestamp));
            blues.push(i);
        }
    }

    println!("{}", reds.len() + blues.len() - 1); // Print the number of nodes in the file
    for vertex in nodes.iter().skip(1) {
        println!("{} {} {}", vertex.0, vertex.1, vertex.2);
    }
}
