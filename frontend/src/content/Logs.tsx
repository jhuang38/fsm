import { Box, Paper, Typography } from "@mui/material";
import { useEffect, useRef, useState } from "react";



export const Logs = () => {
    const [messages, setMessages] = useState<Array<string>>([]);

    const socketConn = useRef<WebSocket>(null);
    useEffect(() => {
        const socket = new WebSocket("ws://localhost:5000/fsm/ws");
        socket.onmessage = (event) => {
            console.log({event})
            setMessages(messages => [...messages, event.data])
        }
        socketConn.current = socket;
        return () => socketConn.current.close()
    }, [])

    return (
        <Box sx = {{
            flexDirection: 'column',
            width: '100%',
            height: '100%'
        }}>
            <Typography variant="h4">File Movement Logs</Typography>
            <Paper elevation={1} sx = {{
                padding: '1em',
                display: 'flex',
                flexDirection: 'column',
                gap: '1em'
            }}>
            {
                messages.length > 0 &&
                    <Box sx = {{
                        flexDirection: 'column',
                        display: 'flex',
                        gap: '0.5em',
                        borderRadius: 1
                    
                    }}>
                        {
                            messages.map(m => {
                                return (
                                    <Typography sx = {{
                                        padding:'1em',
                                        borderRadius: 1,
                                        '&:hover': {
                                            border: 1,
                                            borderColor: 'primary.dark',
                                            
                                        },
                                    }}>{m}</Typography>
                                )
                                
                            })
                        }
                    </Box>
            }
            </Paper>
            
            
            
        </Box>
    )
}