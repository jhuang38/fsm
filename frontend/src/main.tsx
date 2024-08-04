import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import { createTheme, ThemeProvider } from '@mui/material'
import './style.css'


const themeOptions = {
  palette: {
    type: 'light',
    primary: {
      main: '#5e35b1',
      dark: '#4527a0',
    },
    secondary: {
      main: '#f50057',
    },
  },
  typography: {
    fontFamily: 'Montserrat',
    h1: {
      fontSize: 48,
      fontWeight: 500
    },
    h2: {
      fontSize: 42
    },
    h3: {
      fontSize: 36
    },
    h4: {
      fontSize: 30
    },
    h5: {
      fontSize: 24
    },
    h6: {
      fontSize: 18
    },
    body1: {
      fontSize: 12
    },
    body2: {
      fontSize: 12
    }
  },
};

const theme = createTheme(themeOptions);

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThemeProvider theme={theme}>
      <App />
    </ThemeProvider>
      
  </React.StrictMode>,
)
