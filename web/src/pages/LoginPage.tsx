import React, { useState } from 'react';
import { Button, IconButton, Typography, Link, Box } from '@mui/material';
import mySvg from '../assets/images/login-basable.svg';
import arrowBack from '../assets/images/signup-arrowback.svg';
import LoginForm from '../components/forms/LoginForm';
import backImg1 from '../assets/images/signup_back1.svg';

const AuthPage: React.FC = () => {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100vh',
        backgroundColor: 'white',
        position: 'relative',
        overflow: 'hidden'
      }}
    >
      <Box
        sx={{
          position: 'absolute',
          top: -150,
          left: -230
        }}
      >
        <img src={backImg1} width="500px" />
      </Box>
      <Box
        sx={{
          position: 'absolute',
          top: 10,
          left: 1030
        }}
      >
        <img src={backImg1} />
      </Box>
      <Box
        sx={{
          position: 'absolute',
          top: 720,
          left: 620
        }}
      >
        <img src={backImg1} />
      </Box>
      <Box
        sx={{
          position: 'absolute',
          bottom: -200,
          right: -200
        }}
      >
        <img src={backImg1} width="500px" />
      </Box>

      <Box
        sx={{
          width: 400,
          padding: 4,
          borderRadius: 2,
          backgroundColor: '#fff',
          boxShadow: '0 4px 12px rgba(0, 0, 0, 0.1)',
          textAlign: 'center',
          position: 'relative'
        }}
      >
        <Link href="/" style={{ textDecoration: 'none' }}>
          <IconButton sx={{ position: 'absolute', top: 8, left: 8 }}>
            <img src={arrowBack} />
          </IconButton>
        </Link>

        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            margin: '15px'
          }}
        >
          <img src={mySvg} width="50" />
        </Box>

        <Typography
          variant="h4"
          component="h1"
          sx={{ fontWeight: 'bold', color: '#5a5fcf', mb: 1 }}
        >
          Login
        </Typography>

        <Typography variant="body2" color="textSecondary" sx={{ mb: 3 }}>
          Log in to explore, analyze, and succeed.
        </Typography>

        <Button
          variant="outlined"
          sx={{
            width: '100%',
            borderColor: '#dcdcdc',
            color: 'black',
            textTransform: 'none',
            mb: 2,
            borderRadius: 50
          }}
          startIcon={
            <img
              src="https://img.icons8.com/color/16/000000/google-logo.png"
              alt="Google logo"
            />
          }
        >
          Log In with Google
        </Button>

        <Typography
          variant="body2"
          color="textSecondary"
          sx={{ mb: 2, color: 'black' }}
        >
          Or
        </Typography>

        <LoginForm />
      </Box>
    </Box>
  );
};

export default AuthPage;
