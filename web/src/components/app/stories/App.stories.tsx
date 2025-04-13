// web/src/components/app/stories/App.stories.tsx
import {useEffect, useState} from 'react'
import {Meta, StoryObj} from '@storybook/react'
import App from 'src/components/app/App'
import init, {WasmKanbanBoard} from 'bindgen/kandown_wasm'

// Sample markdown for the demo
const sampleMarkdown = `# Properties
- Owner: Text
- Status: Select
	- Backlog
	- In Progress
	- Done

# Views
- Task Board
  Layout: Board
  Group: Status
  Display:
- Data Table
  Layout: Table
  Group: Status

# Cards
- Task 1
  Owner: bob
  Status: In Progress
  This is a description

- Task 2
  Owner: alice
  Status: Backlog
  Another task description

- Task 3
  Owner: charlie
  Status: Done
  A completed task

- Task 4
  Owner: david
  Status: Backlog
  Another task in the backlog`

// Create a wrapper component that handles the WASM initialization
const AppStory = () => {
    const [board, setBoard] = useState<WasmKanbanBoard | null>(null)
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    useEffect(() => {
        const loadWasm = async () => {
            try {
                console.debug('[AppStory] Initializing WASM module...')
                await init()
                console.debug('[AppStory] Creating WasmKanbanBoard...')
                const wasmBoard = new WasmKanbanBoard(sampleMarkdown)
                console.debug('[AppStory] Board created successfully:', wasmBoard)
                setBoard(wasmBoard)
                setLoading(false)
            } catch (err) {
                console.error('[AppStory] Failed to initialize WASM:', err)
                setError(err instanceof Error ? err.message : String(err))
                setLoading(false)
            }
        }

        loadWasm()
    }, [])

    if (loading) {
        return <div style={{padding: 20}}>Loading WASM module...</div>
    }

    if (error) {
        return <div style={{padding: 20, color: 'red'}}>Error loading WASM: {error}</div>
    }

    if (!board) {
        return <div style={{padding: 20}}>Failed to initialize board</div>
    }

    return <App board={() => Promise.resolve(board)} />
}

const meta: Meta<typeof AppStory> = {
    title: 'App/KanbanApp',
    component: AppStory,
    parameters: {
        layout: 'fullscreen',
    },
}

export default meta
type Story = StoryObj<typeof AppStory>

export const Default: Story = {}
