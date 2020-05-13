use crate::{Block, Screen, Wrapper};
use quicksilver::{
    geom::{Rectangle, Vector},
    Result,
};

use async_trait::async_trait;

pub struct Menu {
    level: Vec<Vec<Block>>,
}

impl Menu {
    pub(crate) async fn new(wrapper: &mut Wrapper<'_>) -> Result<Self> {
        Ok(Self {
            level: wrapper.get_level(0).await?,
        })
    }
}

const BLOCK_SIZE: usize = 32;
const BLOCK_SIZE_I32: i32 = 32;

#[async_trait(?Send)]
impl Screen for Menu {
    async fn draw(&mut self, wrapper: &mut crate::Wrapper<'_>) -> quicksilver::Result<()> {
        for (y, line) in self.level.iter().enumerate() {
            for (x, block) in line.iter().enumerate() {
                let block = wrapper.get_block(*block).await?;
                wrapper.gfx.draw_image(
                    &block,
                    Rectangle::new(
                        ((x * BLOCK_SIZE) as i32, (y * BLOCK_SIZE) as i32),
                        (BLOCK_SIZE_I32, BLOCK_SIZE_I32),
                    ),
                );
            }
        }
        Ok(())
    }
    async fn update(
        &mut self,
        wrapper: &mut crate::Wrapper<'_>,
    ) -> quicksilver::Result<Option<Box<dyn Screen>>> {
        Ok(None)
    }
}
