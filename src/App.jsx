// src/App.jsx
import { Box } from '@mui/material'
import FileExplorer from './components/FileExplorer'

function App() {
  return (
    <Box sx={{ height: '100vh', width: '100vw', overflow: 'hidden' }}>
      <FileExplorer />
    </Box>
  )
}

export default App