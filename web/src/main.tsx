// web/src/main.tsx
import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import 'src/main.css'
import App from './components/app/App.tsx'
import init, {WasmKanbanBoard} from 'bindgen'

async function getBoard() {
    await init()
    const board = new WasmKanbanBoard(`# Properties
- Owner: Text
- Status: Select
	- Backlog
	- In Progress
	- Done

# Views
- Task Board
  Layout: Board
  Group: Status
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
  Another task in the backlog`)

    // Make board available globally for debugging
    // @ts-ignore
    window.board = board

    return board
}
const rootElement = document.getElementById('root')
if (!rootElement) {
    throw new Error('Root element not found')
}
createRoot(rootElement).render(
    <StrictMode>
        <App board={getBoard} />
    </StrictMode>,
)
