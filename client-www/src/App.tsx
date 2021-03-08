
import React, {
  useState,
  useEffect
} from 'react';

import {
  Switch,
  Route,
  NavLink,
  Link,
  useParams
} from 'react-router-dom';

import {
  Button
} from '@material-ui/core';

import './App.css';


interface IdParams {
  id: string;
}

interface Board {
  id: string;
  title: string;
  owner: string;
  created_at: number;
}

interface Column {
  id: string;
  board_id: string;
  order: number;
  title: string;
  created_at: number;
}

interface Card {
  id: string;
  column_id: string;
  order: number;
  title: string;
  author: string;
  created_at: number;
}

interface Comment {
  id: string;
  card_id: string;
  contents: string;
  author: string;
  created_at: number;
}

interface Vote {
  id: string;
  card_id: string;
  author: string;
  created_at: number;
}

interface Tag {
  id: string;
  board_id: string;
  title: string;
  created_at: number;
}

interface CardTag {
  id: string;
  card_id: string;
  tag_id: string;
  created_at: number;
}


export default function App() {
  return (
    <div>
      <nav>
        <ul>
          <li>
            <NavLink to="/">Home</NavLink>
          </li>
          <li>
            <NavLink to="/boards">Boards</NavLink>
          </li>

          <li>
            <NavLink to="/about">About</NavLink>
          </li>
          <li>
            <NavLink to="/users">Users</NavLink>
          </li>
        </ul>
      </nav>

      {/* A <Switch> looks through its children <Route>s and
          renders the first one that matches the current URL. */}
      <Switch>
        <Route path="/board/:id">
          <BoardComponent />
        </Route>
        <Route path="/boards">
          <MyBoardsComponent />
        </Route>

        <Route path="/about">
          <AboutComponent />
        </Route>
        <Route path="/users">
          <UsersComponent />
        </Route>
        <Route path="/">
          <HomeComponent />
        </Route>
      </Switch>
    </div>
  );
}

function HomeComponent () {
  return (
    <div>
      <h2>Home</h2>
      <Button variant="contained" color="primary">
        Hello World - Home
      </Button>
    </div>
  );
}

function MyBoardsComponent () {
  const [boards, setBoards] = useState<Board[]>([]);

  // https://www.robinwieruch.de/react-hooks-fetch-data
  //  -- need second arg to avoid infinite loop
  useEffect(() => {
    fetch('/api/boards')
      .then(response => response.json())
      .then(json => setBoards(json))
  }, []);

  return (
    <div>
      <h2>Boards</h2>
      {boards.map((board, index) => (
          <div key={index}>
            <Link to={"/board/" + board.id}>{board.title}</Link>
          </div>
        ))}
    </div>
  );
}

function BoardComponent () {
  const { id } = useParams<IdParams>();
  const [board, setBoard] = useState<Board | null>(null);

  useEffect(() => {
    fetch('/api/boards/' + id)
      .then(response => response.json())
      .then(json => setBoard(json))
  }, []);

  return (
    <div>
      <h2>Board {id}</h2>
      {board
        ? (
          <div>
            <p>{board.title}</p>
            <p>{board.owner}</p>
            <p>{board.created_at}</p>
          </div>
        )
        : <p>Loading...</p>
      }
    </div>
  );
}

function AboutComponent () {
  return (
    <div>
      <h2>About</h2>
      <Button variant="contained" color="secondary">
        Hello World - About
      </Button>
    </div>
  );
}

function UsersComponent () {
  return (
    <div>
      <h2>Users</h2>
      <Button variant="contained" color="default">
        Hello World - Users
      </Button>
    </div>
  );
}
