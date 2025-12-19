import React, { useEffect, useState } from 'react'
import axios from 'axios'
import type { RouteConfig } from './types'

export default function App() {
    const [routes, setRoutes] = useState<RouteConfig[]>([])

    useEffect(() => {
        axios.get('/api/routes').then((res) => setRoutes(res.data || []))
    }, [])

    return (
        <div className="app">
            <header>
                <h1>BFF MVP</h1>
            </header>
            <main>
                <section>
                    <h2>Configured routes</h2>
                    <pre>{JSON.stringify(routes, null, 2)}</pre>
                </section>
            </main>
        </div>
    )
}
