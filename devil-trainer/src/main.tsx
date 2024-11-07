import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';
import './App.css';  // Import the CSS file to apply styles

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  document.getElementById('root')  // This is where the App component will be rendered
);
