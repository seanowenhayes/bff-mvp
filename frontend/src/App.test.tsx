import { render, screen, waitFor } from '@testing-library/react'
import App from './App'
import axios from 'axios'
import { expect, test, vi } from 'vitest'

vi.mock('axios', () => ({ default: { get: vi.fn(() => Promise.resolve({ data: [] })) } }))

test('renders header and fetches routes', async () => {
    render(<App />)
    expect(screen.getByText('BFF MVP')).toBeInTheDocument()
    await waitFor(() => expect((axios as any).get).toHaveBeenCalledWith('/api/routes'))
})
