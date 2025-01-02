import React, { useState, useEffect } from 'react';

const App = () => {
  const [message, setMessage] = useState('');

  useEffect(() => {
    fetch('http://localhost:8080/')
      .then((response) => response.text())
      .then((data) => setMessage(data))
      .catch((error) => console.error('Error fetching data:', error));
  }, []);

  return (
    <div>
      <h1>Hello World Appv2</h1>
      <p>{message ? message : 'Loading...'}</p>
    </div>
  );
};

export default App;