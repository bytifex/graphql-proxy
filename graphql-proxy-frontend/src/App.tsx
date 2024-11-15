import { useState } from 'react';
import './App.css';
import Message from './components/Message';
import MessageData from './model/message_data';

function App() {
  const [messages, setMessages] = useState<MessageData[]>(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]);

  return <>
    {
      messages.map((message) => {
        return <Message message={message} />;
      })
    }
  </>;
}

export default App;
