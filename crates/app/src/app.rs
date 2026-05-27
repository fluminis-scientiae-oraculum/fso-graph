use std::{path::PathBuf, time::Instant};

use anyhow::Result;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use graph::prelude::*;
use log::info;

use crate::runner::gen_runner;

mod loading;
mod runner;
mod serialize;
mod triangle_count;

gen_runner!(directed+unweighted: page_rank, graph::page_rank::page_rank, PageRankConfig);
gen_runner!(directed+unweighted: wcc, graph::wcc::wcc_afforest_dss, WccConfig);
gen_runner!(directed+weighted: sssp, graph::sssp::delta_stepping, DeltaSteppingConfig, f32);

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    match args.algorithm {
        Algorithm::PageRank { config } => page_rank::run(args.args, config)?,
        Algorithm::Sssp { config } => sssp::run(args.args, config)?,
        Algorithm::TriangleCount { relabel } => triangle_count::triangle_count(args.args, relabel)?,
        Algorithm::Wcc { config } => wcc::run(args.args, config)?,
        Algorithm::Loading {
            undirected,
            weighted,
        } => loading::loading(args.args, undirected, weighted)?,
        Algorithm::Serialize { output, undirected } => {
            serialize::serialize(args.args, undirected, output)?
        }
    }

    Ok(())
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, propagate_version = true)]
struct Args {
    #[command(flatten)]
    args: CommonArgs,

    #[command(subcommand)]
    algorithm: Algorithm,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[derive(Debug, clap::Args)]
struct CommonArgs {
    #[arg(short, long, value_parser)]
    path: PathBuf,

    #[arg(short, long, value_enum, default_value_t = FileFormat::EdgeList)]
    format: FileFormat,

    #[arg(short, long, value_enum, default_value_t = GraphFormat::CompressedSparseRow)]
    graph: GraphFormat,

    #[arg(long)]
    use_32_bit: bool,

    #[arg(short, long, default_value_t = 1)]
    runs: usize,

    #[arg(short, long, default_value_t = 5)]
    warmup_runs: usize,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum GraphFormat {
    CompressedSparseRow,
    AdjacencyList,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum FileFormat {
    EdgeList,
    Graph500,
}

#[derive(clap::Subcommand, Debug)]
enum Algorithm {
    PageRank {
        #[command(flatten)]
        config: PageRankConfig,
    },
    Sssp {
        #[command(flatten)]
        config: DeltaSteppingConfig,
    },
    TriangleCount {
        #[arg(long)]
        relabel: bool,
    },

    Wcc {
        #[command(flatten)]
        config: WccConfig,
    },
    Loading {
        /// Load the graph as undirected.
        #[arg(long)]
        undirected: bool,
        /// Load the graph as weighted.
        #[arg(long)]
        weighted: bool,
    },
    Serialize {
        /// Path to serialize graph to.
        #[arg(short, long, value_parser)]
        output: PathBuf,
        /// Load the graph as undirected.
        #[arg(long)]
        undirected: bool,
    },
}

pub(crate) fn time(runs: usize, warmup_runs: usize, f: impl Fn()) {
    for run in 1..=warmup_runs {
        let start = Instant::now();
        f();
        let took = start.elapsed();

        info!(
            "Warm-up run {} of {} finished in {:.6?}",
            run, warmup_runs, took,
        );
    }

    let mut durations = vec![];

    for run in 1..=runs {
        let start = Instant::now();
        f();
        let took = start.elapsed();
        durations.push(took);

        info!("Run {} of {} finished in {:.6?}", run, runs, took,);
    }

    let total = durations
        .into_iter()
        .reduce(|a, b| a + b)
        .unwrap_or_default();

    info!("Average runtime: {:?}", total / runs as u32);
}
