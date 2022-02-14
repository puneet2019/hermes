use crate::bootstrap::single::bootstrap_single_node;
use crate::chain::builder::ChainBuilder;
use crate::error::Error;
use crate::framework::base::HasOverrides;
use crate::framework::base::{run_basic_test, BasicTest, TestConfigOverride};
use crate::framework::binary::node::NodeConfigOverride;
use crate::types::config::TestConfig;
use crate::types::single::node::FullNode;
use crate::util::array::try_into_array;

pub fn run_nary_node_test<Test, Overrides, const SIZE: usize>(test: &Test) -> Result<(), Error>
where
    Test: NaryNodeTest<SIZE>,
    Test: HasOverrides<Overrides = Overrides>,
    Overrides: NodeConfigOverride + TestConfigOverride,
{
    run_basic_test(&RunNaryNodeTest { test })
}

pub trait NaryNodeTest<const SIZE: usize> {
    fn run(&self, config: &TestConfig, nodes: [FullNode; SIZE]) -> Result<(), Error>;
}

pub struct RunNaryNodeTest<'a, Test, const SIZE: usize> {
    pub test: &'a Test,
}

impl<'a, Test, Overrides, const SIZE: usize> BasicTest for RunNaryNodeTest<'a, Test, SIZE>
where
    Test: NaryNodeTest<SIZE>,
    Test: HasOverrides<Overrides = Overrides>,
    Overrides: NodeConfigOverride,
{
    fn run(&self, config: &TestConfig, builder: &ChainBuilder) -> Result<(), Error> {
        let mut nodes = Vec::new();
        let mut node_processes = Vec::new();

        for i in 0..SIZE {
            let node = bootstrap_single_node(builder, &format!("{}", i), |config| {
                self.test.get_overrides().modify_node_config(config)
            })?;

            node_processes.push(node.process.clone());
            nodes.push(node);
        }

        self.test.run(config, try_into_array(nodes)?)?;

        Ok(())
    }
}

impl<'a, Test, Overrides, const SIZE: usize> HasOverrides for RunNaryNodeTest<'a, Test, SIZE>
where
    Test: HasOverrides<Overrides = Overrides>,
{
    type Overrides = Overrides;

    fn get_overrides(&self) -> &Self::Overrides {
        self.test.get_overrides()
    }
}