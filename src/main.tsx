import React from 'react'
import {BrowserRouter, Route, Routes, useNavigate} from 'react-router-dom';
import ReactDOM from 'react-dom/client'
import $ from 'jquery'
import {Link, NextUIProvider} from "@nextui-org/react";

import {Navbar, NavbarBrand, NavbarContent, NavbarItem} from "@nextui-org/navbar";

import "./assets/scss/index.scss"
import Home from "./assets/pages/Home.tsx";
import About from "./assets/pages/About.tsx";
import ThemeSwitcher, {applyTheme} from "./assets/components/ThemeSwitcher.tsx";

applyTheme()

ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <PageContent/>
        </BrowserRouter>
    </React.StrictMode>,
)


function PageContent() {
    const navigate = useNavigate();
    return (
        <NextUIProvider navigate={navigate}>
            <Nav/>
            <Routes>
                <Route>
                    <Route path="/" element={<Home/>}/>
                    <Route path="/about" element={<About/>}/>
                </Route>
            </Routes>
        </NextUIProvider>
    );
}


function Nav() {
    return (
        <Navbar>
            <NavbarBrand>
                <p className="font-bold text-inherit">Example</p>
            </NavbarBrand>
            <NavbarContent className="hidden sm:flex gap-4" justify="center">
                <NavbarItem>
                    <Link color="foreground" href="/">
                        Home
                    </Link>
                </NavbarItem>
                <NavbarItem>
                    <Link href="/about">
                        About
                    </Link>
                </NavbarItem>
            </NavbarContent>
            <NavbarContent justify="end">
                <NavbarItem>
                    <ThemeSwitcher/>
                </NavbarItem>
            </NavbarContent>
        </Navbar>)
}