// src/components/FileExplorer.jsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/shell';
import {
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  TextField,
  InputAdornment,
  IconButton,
  Breadcrumbs,
  Link,
  Typography,
  CircularProgress,
  Box,
  Tooltip,
  Skeleton
} from '@mui/material';
import {
  Folder as FolderIcon,
  InsertDriveFile as FileIcon,
  Search as SearchIcon,
  Refresh as RefreshIcon,
  Home as HomeIcon,
  ArrowUpward as UpIcon
} from '@mui/icons-material';

const FileExplorer = () => {
  const [currentDir, setCurrentDir] = useState('C:\\');
  const [files, setFiles] = useState([]);
  const [loading, setLoading] = useState(true);
  const [initializing, setInitializing] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchExtension, setSearchExtension] = useState('');
  const [breadcrumbs, setBreadcrumbs] = useState(['C:']);

  useEffect(() => {
    const initializeApp = async () => {
      setInitializing(true);
      try {
        // Load the initial directory
        await loadDirectory('C:\\');
      } catch (error) {
        console.error('Initialization error:', error);
      } finally {
        setInitializing(false);
      }
    };
    
    initializeApp();
  }, []);
  
  useEffect(() => {
    if (!initializing) {
      setBreadcrumbs(currentDir.split('\\').filter(Boolean));
      loadDirectory(currentDir);
    }
  }, [currentDir, initializing]);

  const loadDirectory = async (path) => {
    setLoading(true);
    try {
      const result = await invoke('list_directory_contents', { path });
      setFiles(result);
    } catch (error) {
      console.error('Error loading directory:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async () => {
    setLoading(true);
    try {
      const result = await invoke('search_files', {
        name: searchQuery,
        extension: searchExtension,
      });
      setFiles(result);
    } catch (error) {
      console.error('Error searching files:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleFileClick = async (file) => {
    if (!file.extension) {
      // It's a directory
      setCurrentDir(file.path);
    } else {
      // It's a file - try to open it
      try {
        await invoke('open_file', { path: file.path });
      } catch (error) {
        console.error('Error opening file:', error);
        // Optional: show error to user
      }
    }
  };

  const navigateTo = (index) => {
    const newPath = breadcrumbs.slice(0, index + 1).join('\\') + '\\';
    setCurrentDir(newPath);
  };

  const goUp = () => {
    const newPath = breadcrumbs.slice(0, -1).join('\\') + '\\';
    if (newPath) {
      setCurrentDir(newPath);
    }
  };

  const goHome = () => {
    setCurrentDir('C:\\');
  };

  const refresh = () => {
    loadDirectory(currentDir);
  };

  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  };

  if (initializing) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" height="100vh">
        <CircularProgress />
        <Typography variant="body1" style={{ marginLeft: '16px' }}>
          Initializing file index...
        </Typography>
      </Box>
    );
  }

  return (
    <div style={{ padding: '20px', height: '100vh', display: 'flex', flexDirection: 'column' }}>
      <div style={{ marginBottom: '20px', display: 'flex', gap: '10px' }}>
        <TextField
          fullWidth
          variant="outlined"
          placeholder="Search files..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon />
              </InputAdornment>
            ),
            endAdornment: (
              <InputAdornment position="end">
                <IconButton onClick={handleSearch} edge="end">
                  <SearchIcon />
                </IconButton>
              </InputAdornment>
            ),
          }}
        />
        <TextField
          variant="outlined"
          placeholder="Extension"
          value={searchExtension}
          onChange={(e) => setSearchExtension(e.target.value)}
          style={{ width: '150px' }}
        />
      </div>

      <div style={{ marginBottom: '20px', display: 'flex', alignItems: 'center', gap: '10px' }}>
        <Tooltip title="Home">
          <IconButton onClick={goHome}>
            <HomeIcon />
          </IconButton>
        </Tooltip>
        <Tooltip title="Up">
          <IconButton onClick={goUp} disabled={breadcrumbs.length <= 1}>
            <UpIcon />
          </IconButton>
        </Tooltip>
        <Tooltip title="Refresh">
          <IconButton onClick={refresh}>
            <RefreshIcon />
          </IconButton>
        </Tooltip>
        <Breadcrumbs aria-label="breadcrumb" style={{ flexGrow: 1 }}>
          {breadcrumbs.map((crumb, index) => (
            <Link
              key={index}
              color="inherit"
              href="#"
              onClick={(e) => {
                e.preventDefault();
                navigateTo(index);
              }}
            >
              {crumb}
            </Link>
          ))}
        </Breadcrumbs>
      </div>

      <TableContainer component={Paper} style={{ flexGrow: 1, overflow: 'auto' }}>
        <Table stickyHeader>
          <TableHead>
            <TableRow>
              <TableCell>Name</TableCell>
              <TableCell>Directory</TableCell>
              <TableCell align="right">Size</TableCell>
              <TableCell>Modified</TableCell>
              <TableCell>Type</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {loading ? (
              Array.from({ length: 10 }).map((_, index) => (
                <TableRow key={index}>
                  <TableCell colSpan={4}>
                    <Skeleton variant="text" />
                  </TableCell>
                </TableRow>
              ))
            ) : files.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} align="center">
                  No files found
                </TableCell>
              </TableRow>
            ) : (
              files.map((file) => (
                <TableRow
                  key={file.path}
                  hover
                  onClick={() => handleFileClick(file)}
                  style={{ cursor: 'pointer' }}
                >
                  <TableCell>
                    <Box display="flex" alignItems="center">
                      {file.extension ? (
                        <FileIcon color="action" style={{ marginRight: '8px' }} />
                      ) : (
                        <FolderIcon color="primary" style={{ marginRight: '8px' }} />
                      )}
                      {file.name}
                    </Box>
                  </TableCell>
                  <TableCell>
                    {file.path.replace(/\\/g, '\\').replace(/\\[^\\]+\\?$/, '')}
                  </TableCell>
                  <TableCell align="right">
                    {file.extension ? formatFileSize(file.size) : '-'}
                  </TableCell>
                  <TableCell>{formatDate(file.modified)}</TableCell>
                  <TableCell>{file.extension || 'Folder'}</TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </div>
  );
};

export default FileExplorer;