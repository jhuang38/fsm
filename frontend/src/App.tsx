import { Menu } from "@mui/icons-material";
import { AppBar, Box, Drawer, IconButton, List, ListItem, ListItemButton, Typography } from "@mui/material";
import { useState } from "react";
import {Logs} from "./content/Logs";


function App() {
  const [drawOpen, setDrawerOpen] = useState(false);
  

  return (
    <Box sx = {{
      flexGrow: 1,
      
    }}>
      <AppBar position='static' sx = {{
        flexDirection: 'row',
        padding: '1em',
        gap: '2em'
      }}>
        <IconButton
          size="large"
          edge="start"
          color="inherit"
          aria-label="menu"
          onClick={() => {
            setDrawerOpen(true);
          }}
        >
          <Menu/>
        </IconButton>
        <Typography variant="h3">FSM Management Dashboard</Typography>
      </AppBar>
      <Box sx = {{
        flexGrow: 1,
        padding: '2em'
      }}>
        <Logs/>
      </Box>
      <Drawer open={drawOpen} onClose={() => {
        setDrawerOpen(false);
      }}>
        <List sx = {{
          padding: 0
        }}>
          <ListItem key={'home'} sx = {{
            padding: 0
          }}>
            <ListItemButton>
              <Box>
                <Typography sx = {{
                fontSize: '1em',
                width: 250
              }}>Home</Typography>
              </Box>
            </ListItemButton>      
          </ListItem>
          <ListItem key={'filters'} sx = {{
            padding: 0
          }}>
            <ListItemButton>
              <Box>
                <Typography sx = {{
                fontSize: '1em',
                width: 250
              }}>Filter Management</Typography>
              </Box>
            </ListItemButton>      
          </ListItem>
          <ListItem key={'jobs'} sx = {{
            padding: 0
          }}>
            <ListItemButton>
              <Box>
                <Typography sx = {{
                fontSize: '1em',
                width: 250
              }}>Job Management</Typography>
              </Box>
            </ListItemButton>      
          </ListItem>
          <ListItem key={'config'} sx = {{
            padding: 0
          }}>
            <ListItemButton>
              <Box >
                <Typography sx = {{
                fontSize: '1em',
                width: 250
              }}>Config Settings</Typography>
              </Box>
            </ListItemButton>      
          </ListItem>
        </List>
        
      </Drawer>
    </Box>
  );
}

export default App
